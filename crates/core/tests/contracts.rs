use kitcontrol_core::{
    Request, dispatch, dispatch_json,
    edit::{EditOperation, edit},
    model::{DeviceIdentity, Document, Snapshot},
    storage,
    transfer::{self, DeviceTransport, Fault, Simulator},
};
use serde_json::{Value, json};
use std::{fs, path::Path};

fn document() -> Document {
    serde_json::from_str(include_str!("fixtures/document.json")).unwrap()
}

fn snapshot() -> Snapshot {
    Snapshot {
        identity: DeviceIdentity {
            device_id: "test-device".into(),
            profile_id: "kitcontrol.synthetic.v1".into(),
            simulated: true,
        },
        document: document(),
    }
}

fn renamed_document() -> Document {
    let mut expected = document();
    expected.kits.get_mut("amber").unwrap().name = "Edited amber".into();
    expected
}

#[test]
fn copy_pad_preserves_complete_layer_and_unrelated_kit_data() {
    let original = document();
    let actual = edit(
        &original,
        EditOperation::CopyPad {
            source_kit_id: "amber".into(),
            source_pad: "snare.head".into(),
            target_kit_id: "blue".into(),
            target_pad: "kick.head".into(),
        },
    )
    .unwrap();
    let expected_pad = json!({
        "instrument":"synth.snare.01", "tuning":-17, "muffling":3,
        "volume":72, "pan":-11, "sub_instrument":"synth.tom.01", "layer_level":29
    });
    assert_eq!(
        serde_json::to_value(&actual.kits["blue"].pads["kick.head"]).unwrap(),
        expected_pad
    );
    assert_eq!(
        serde_json::to_value(&actual.kits["blue"].effects).unwrap(),
        json!({"test-room":{"level":7,"enabled":false}})
    );
    assert_eq!(
        serde_json::to_value(&actual.kits["blue"].opaque).unwrap(),
        json!({"target-extension":[42,0,201]})
    );
    assert_eq!(actual.kits["amber"], original.kits["amber"]);
    assert_eq!(actual.kits["spare"], original.kits["spare"]);
    assert_eq!(original, document());
}

#[test]
fn copy_kit_preserves_effects_layers_opaque_and_overwritten_library_kit() {
    let original = document();
    let actual = edit(
        &original,
        EditOperation::CopyKit {
            source_kit_id: "amber".into(),
            target_slot: 2,
            new_kit_id: "amber-copy".into(),
        },
    )
    .unwrap();
    assert_eq!(actual.slots, ["amber", "amber-copy"]);
    assert_eq!(actual.kits["amber-copy"], document().kits["amber"]);
    assert_eq!(actual.kits["blue"], document().kits["blue"]);
    assert_eq!(actual.kits["spare"], document().kits["spare"]);
    assert_eq!(actual.kits.len(), 4);
    assert_eq!(actual.setlists[0].slots, [1, 2, 1]);
    assert_eq!(original, document());
}

#[test]
fn reorder_remaps_one_based_setlist_references_to_the_same_kit_identity() {
    let actual = edit(
        &document(),
        EditOperation::ReorderKits {
            order: vec!["blue".into(), "amber".into()],
        },
    )
    .unwrap();
    assert_eq!(actual.slots, ["blue", "amber"]);
    assert_eq!(actual.setlists[0].slots, [2, 1, 2]);
    assert_eq!(actual.kits, document().kits);
}

#[test]
fn rejected_parameter_edits_do_not_mutate_the_source() {
    for (parameter, value, expected_error) in [
        ("tuning", json!(-101), "out_of_range"),
        ("tuning", json!(101), "out_of_range"),
        ("muffling", json!(-1), "out_of_range"),
        ("volume", json!(101), "out_of_range"),
        ("pan", json!(31), "out_of_range"),
        ("layer_level", json!(101), "out_of_range"),
        ("volume", json!(1.5), "invalid_value"),
        ("volume", json!(2147483648_i64), "invalid_value"),
        (
            "instrument",
            json!("test.unknown"),
            "unsupported_instrument",
        ),
        ("sub_instrument", json!(false), "invalid_value"),
        ("unrecognized", json!(3), "unknown_parameter"),
    ] {
        let source = document();
        let error = edit(
            &source,
            EditOperation::SetParameter {
                kit_id: "amber".into(),
                pad: "snare.head".into(),
                parameter: parameter.into(),
                value,
            },
        )
        .unwrap_err();
        assert_eq!(error.code, expected_error, "parameter {parameter}");
        assert_eq!(source, document());
    }
}

#[test]
fn invalid_slot_duplicate_identity_and_invalid_order_are_rejected() {
    for operation in [
        json!({"kind":"copy_kit","source_kit_id":"amber","target_slot":0,"new_kit_id":"copy"}),
        json!({"kind":"copy_kit","source_kit_id":"amber","target_slot":3,"new_kit_id":"copy"}),
        json!({"kind":"copy_kit","source_kit_id":"amber","target_slot":1,"new_kit_id":"blue"}),
        json!({"kind":"reorder_kits","order":["amber","amber"]}),
        json!({"kind":"reorder_kits","order":["amber","spare"]}),
    ] {
        let source = document();
        assert!(edit(&source, serde_json::from_value(operation).unwrap()).is_err());
        assert_eq!(source, document());
    }
}

#[test]
fn storage_roundtrip_is_stable_and_preserves_unoccupied_library_kits() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("workspace");
    assert_eq!(storage::save(&path, &document()).unwrap(), 4);
    assert_eq!(storage::load(&path).unwrap(), document());
    let first = workspace_bytes(&path);
    assert_eq!(storage::save(&path, &document()).unwrap(), 4);
    assert_eq!(workspace_bytes(&path), first);
    assert!(!path.join(".kitcontrol-pending.json").exists());
    let manifest: Value =
        serde_json::from_slice(&fs::read(path.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(manifest["kits"], json!(["amber", "blue", "spare"]));
    assert_eq!(manifest["slots"], json!(["amber", "blue"]));
}

fn workspace_bytes(path: &Path) -> Vec<Vec<u8>> {
    [
        "manifest.json",
        "kits/amber.json",
        "kits/blue.json",
        "kits/spare.json",
    ]
    .iter()
    .map(|name| fs::read(path.join(name)).unwrap())
    .collect()
}

#[test]
fn save_rejects_dirty_destination_and_a_foreign_document_without_overwriting() {
    let root = tempfile::tempdir().unwrap();
    let dirty = root.path().join("dirty");
    fs::create_dir(&dirty).unwrap();
    fs::write(dirty.join("owner-data.txt"), "keep this file").unwrap();
    assert_eq!(
        storage::save(&dirty, &document()).unwrap_err().code,
        "unsafe_destination"
    );
    assert_eq!(
        fs::read_to_string(dirty.join("owner-data.txt")).unwrap(),
        "keep this file"
    );
    assert!(!dirty.join("manifest.json").exists());

    let workspace = root.path().join("workspace");
    storage::save(&workspace, &document()).unwrap();
    let before = workspace_bytes(&workspace);
    let mut foreign = document();
    foreign.document_id = "different-test-archive".into();
    assert_eq!(
        storage::save(&workspace, &foreign).unwrap_err().code,
        "incompatible_document"
    );
    assert_eq!(workspace_bytes(&workspace), before);
    assert!(!workspace.join(".kitcontrol-pending.json").exists());
}

#[test]
fn interrupted_save_requires_explicit_recovery_and_uses_the_pending_document() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path();
    storage::save(path, &document()).unwrap();
    let pending = renamed_document();
    fs::write(
        path.join(".kitcontrol-pending.json"),
        serde_json::to_vec(&pending).unwrap(),
    )
    .unwrap();
    assert_eq!(storage::load(path).unwrap_err().code, "incomplete_save");
    assert_eq!(
        storage::save(path, &document()).unwrap_err().code,
        "incomplete_save"
    );
    assert_eq!(storage::recover(path).unwrap(), 4);
    assert_eq!(storage::load(path).unwrap(), pending);
    assert!(!path.join(".kitcontrol-pending.json").exists());
}

#[test]
fn invalid_pending_document_is_not_committed_or_removed_by_recovery() {
    let root = tempfile::tempdir().unwrap();
    storage::save(root.path(), &document()).unwrap();
    let before = workspace_bytes(root.path());
    let mut pending: Value = serde_json::from_str(include_str!("fixtures/document.json")).unwrap();
    pending["slots"] = json!(["missing-kit"]);
    let journal = root.path().join(".kitcontrol-pending.json");
    fs::write(&journal, serde_json::to_vec(&pending).unwrap()).unwrap();
    assert_eq!(
        storage::recover(root.path()).unwrap_err().code,
        "invalid_slots"
    );
    assert_eq!(workspace_bytes(root.path()), before);
    assert!(journal.exists());
}

#[cfg(unix)]
#[test]
fn workspace_load_and_save_reject_symbolic_links_at_key_paths() {
    use std::os::unix::fs::symlink;

    for relative in [
        ".kitcontrol.lock",
        "manifest.json",
        "kits",
        "kits/amber.json",
    ] {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("workspace");
        storage::save(&path, &document()).unwrap();
        let target = path.join(relative);
        let outside = root.path().join("outside");
        fs::rename(&target, &outside).unwrap();
        symlink(&outside, &target).unwrap();
        assert_eq!(
            storage::load(&path).unwrap_err().code,
            "unsafe_path",
            "load {relative}"
        );
        assert_eq!(
            storage::save(&path, &renamed_document()).unwrap_err().code,
            "unsafe_path",
            "save {relative}"
        );
    }
    let root = tempfile::tempdir().unwrap();
    let actual = root.path().join("actual");
    storage::save(&actual, &document()).unwrap();
    let alias = root.path().join("alias");
    symlink(&actual, &alias).unwrap();
    assert_eq!(storage::load(&alias).unwrap_err().code, "unsafe_path");
    assert_eq!(
        storage::save(&alias, &document()).unwrap_err().code,
        "unsafe_path"
    );
}

#[cfg(unix)]
#[test]
fn recovery_rejects_a_symbolic_link_journal_without_writing_its_target() {
    use std::os::unix::fs::symlink;
    let root = tempfile::tempdir().unwrap();
    let workspace = root.path().join("workspace");
    storage::save(&workspace, &document()).unwrap();
    let before = workspace_bytes(&workspace);
    let outside = root.path().join("pending.json");
    let bytes = serde_json::to_vec(&renamed_document()).unwrap();
    fs::write(&outside, &bytes).unwrap();
    symlink(&outside, workspace.join(".kitcontrol-pending.json")).unwrap();
    assert_eq!(
        storage::recover(&workspace).unwrap_err().code,
        "unsafe_path"
    );
    assert_eq!(fs::read(&outside).unwrap(), bytes);
    assert_eq!(workspace_bytes(&workspace), before);
}

#[test]
fn plan_records_exact_reviewable_change_and_rejects_forged_metadata() {
    let baseline = snapshot();
    let plan = transfer::plan(&baseline, &renamed_document()).unwrap();
    assert_eq!(
        serde_json::to_value(&plan.changes).unwrap(),
        json!([
            {"path":"/kits/amber/name","before":"Amber test kit","after":"Edited amber"}
        ])
    );
    for forged_field in [
        "baseline_hash",
        "desired_hash",
        "changes",
        "format_version",
        "desired",
    ] {
        let mut forged = plan.clone();
        match forged_field {
            "baseline_hash" => forged.baseline_hash = "forged".into(),
            "desired_hash" => forged.desired_hash = "forged".into(),
            "changes" => forged.changes.clear(),
            "format_version" => forged.format_version = 2,
            _ => forged.desired.kits.get_mut("amber").unwrap().name = "Unreviewed change".into(),
        }
        let mut simulator = Simulator::new(baseline.clone()).unwrap();
        assert_eq!(
            transfer::apply(&mut simulator, &forged, Fault::None)
                .unwrap_err()
                .code,
            "invalid_plan"
        );
        assert_eq!(simulator.read().unwrap(), baseline);
    }
}

#[test]
fn stale_baseline_and_wrong_device_are_conflicts_without_a_write() {
    let plan = transfer::plan(&snapshot(), &renamed_document()).unwrap();
    let mut stale = snapshot();
    stale.document.kits.get_mut("blue").unwrap().name = "External change".into();
    let mut wrong_device = snapshot();
    wrong_device.identity.device_id = "different-device".into();
    for device in [stale, wrong_device] {
        let response = dispatch(Request::SimulateApply {
            device: device.clone(),
            plan: Box::new(plan.clone()),
            fault: Fault::None,
        });
        assert_eq!(response.exit_code(), 4);
        let result = response.result.unwrap();
        assert_eq!(result["status"], "conflict");
        assert_eq!(result["written"], 0);
        assert_eq!(result["verified"], 0);
        assert_eq!(result["snapshot"], serde_json::to_value(device).unwrap());
    }
}

#[test]
fn apply_verifies_success_and_reports_disconnect_and_mismatch_as_non_success() {
    for (fault, status, exit_code, written, verified) in [
        (Fault::None, "verified", 0, 1, 1),
        (Fault::Disconnect, "failed", 4, 0, 0),
        (Fault::ReadbackMismatch, "partial", 4, 1, 0),
    ] {
        let plan = transfer::plan(&snapshot(), &renamed_document()).unwrap();
        let response = dispatch(Request::SimulateApply {
            device: snapshot(),
            plan: Box::new(plan),
            fault,
        });
        assert_eq!(response.exit_code(), exit_code);
        assert!(response.ok);
        let result = response.result.unwrap();
        assert_eq!(result["status"], status);
        assert_eq!(result["total"], 1);
        assert_eq!(result["written"], written);
        assert_eq!(result["verified"], verified);
        let expected_name = match fault {
            Fault::None => "Edited amber",
            Fault::Disconnect => "Amber test kit",
            Fault::ReadbackMismatch => "Injected mismatch",
        };
        assert_eq!(
            result["snapshot"]["document"]["kits"]["amber"]["name"],
            expected_name
        );
        if fault == Fault::ReadbackMismatch {
            assert_eq!(
                result["changes"],
                json!([
                    {"path":"/kits/amber/name","before":"Injected mismatch","after":"Edited amber"}
                ])
            );
        } else {
            assert_eq!(result["changes"], json!([]));
        }
    }
}

#[test]
fn no_op_apply_is_verified_without_writing() {
    let plan = transfer::plan(&snapshot(), &document()).unwrap();
    let response = dispatch(Request::SimulateApply {
        device: snapshot(),
        plan: Box::new(plan),
        fault: Fault::Disconnect,
    });
    assert_eq!(response.exit_code(), 0);
    let result = response.result.unwrap();
    assert_eq!(result["status"], "verified");
    assert_eq!(result["total"], 0);
    assert_eq!(result["written"], 0);
    assert_eq!(
        result["snapshot"],
        serde_json::to_value(snapshot()).unwrap()
    );
}

#[test]
fn malformed_and_unknown_request_fields_produce_structured_errors() {
    for request in [
        "{",
        "{\"action\":\"devices\",\"unexpected\":true}",
        "{\"action\":\"demo\",\"unexpected\":true}",
    ] {
        let response = dispatch_json(request);
        assert_eq!(response.exit_code(), 2);
        assert!(!response.ok);
        assert!(response.result.is_none());
        assert_eq!(response.error.unwrap().code, "invalid_request");
    }
}

#[test]
fn explicit_api_version_one_and_legacy_omission_share_the_same_response_contract() {
    for version in [None, Some(json!(1))] {
        let mut input = json!({"action":"validate","document":document()});
        if let Some(version) = version {
            input["api_version"] = version;
        }
        let response = dispatch_json(&input.to_string());
        assert_eq!(response.exit_code(), 0);
        assert_eq!(
            serde_json::to_value(response).unwrap(),
            json!({"api_version":1,"ok":true,"result":{"valid":true}})
        );
    }
}

#[test]
fn unsupported_or_noninteger_api_versions_are_structured_failures() {
    for version in [
        json!(0),
        json!(2),
        json!(-1),
        json!(1.0),
        json!("1"),
        Value::Null,
    ] {
        let input = json!({"action":"validate","api_version":version,"document":document()});
        let response = dispatch_json(&input.to_string());
        assert_eq!(response.exit_code(), 2);
        assert_eq!(
            serde_json::to_value(response).unwrap(),
            json!({"api_version":1,"ok":false,"error":{
                "code":"unsupported_version","message":"Only request API version 1 is supported"
            }})
        );
    }
}

#[test]
fn oversized_save_fails_before_replacing_workspace_files_or_creating_a_journal() {
    let root = tempfile::tempdir().unwrap();
    storage::save(root.path(), &document()).unwrap();
    let before = workspace_bytes(root.path());
    let mut oversized = document();
    // The payload alone reaches the file limit; JSON framing makes it exceed it.
    oversized.kits.get_mut("amber").unwrap().effects.insert(
        "test-large-blob".into(),
        Value::String("x".repeat(16 * 1024 * 1024)),
    );
    assert_eq!(
        storage::save(root.path(), &oversized).unwrap_err().code,
        "file_too_large"
    );
    assert_eq!(workspace_bytes(root.path()), before);
    assert!(!root.path().join(".kitcontrol-pending.json").exists());
}

#[test]
fn mismatch_fault_remains_unverified_when_desired_name_matches_the_fault_marker() {
    let mut desired = document();
    desired.kits.get_mut("amber").unwrap().name = "Injected mismatch".into();
    let plan = transfer::plan(&snapshot(), &desired).unwrap();
    let response = dispatch(Request::SimulateApply {
        device: snapshot(),
        plan: Box::new(plan),
        fault: Fault::ReadbackMismatch,
    });
    assert_eq!(response.exit_code(), 4);
    assert!(response.ok);
    let report = response.result.unwrap();
    assert_eq!(report["status"], "partial");
    assert_eq!(report["written"], 1);
    assert_eq!(report["verified"], 0);
    assert_eq!(report["total"], 1);
    assert_eq!(
        report["changes"],
        json!([{
            "path":"/kits/amber/name",
            "before":"Injected mismatch 2",
            "after":"Injected mismatch"
        }])
    );
}

#[test]
fn concurrent_json_creation_preserves_exactly_one_complete_winning_value() {
    use std::sync::{Arc, Barrier};
    use std::thread;

    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("reviewed-plan.json");
    let barrier = Arc::new(Barrier::new(2));
    let handles: Vec<_> = [
        json!({"writer":"first","reviewed":[7,19,31]}),
        json!({"writer":"second","reviewed":[2,13,29]}),
    ]
    .into_iter()
    .map(|value| {
        let path = path.clone();
        let barrier = Arc::clone(&barrier);
        thread::spawn(move || {
            barrier.wait();
            let result = storage::create_json(&path, &value);
            (value, result)
        })
    })
    .collect();
    let outcomes: Vec<_> = handles
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .collect();
    let winners: Vec<_> = outcomes
        .iter()
        .filter(|(_, result)| result.is_ok())
        .collect();
    let failures: Vec<_> = outcomes
        .iter()
        .filter(|(_, result)| result.is_err())
        .collect();
    assert_eq!(winners.len(), 1);
    assert_eq!(failures.len(), 1);
    assert_eq!(
        failures[0].1.as_ref().unwrap_err().code,
        "destination_exists"
    );
    let persisted: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(persisted, winners[0].0);
    assert_eq!(fs::read_dir(root.path()).unwrap().count(), 1);
}

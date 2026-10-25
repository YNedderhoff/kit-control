use serde_json::{Value, json};
use std::{
    fs,
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

// This original test document is shared only by integration tests.
fn document() -> Value {
    serde_json::from_str(include_str!("../../core/tests/fixtures/document.json")).unwrap()
}

fn snapshot() -> Value {
    json!({
        "identity":{"device_id":"cli-test-device","profile_id":"kitcontrol.synthetic.v1","simulated":true},
        "document":document()
    })
}

fn edited_document() -> Value {
    let mut expected = document();
    expected["kits"]["amber"]["name"] = json!("CLI edited amber");
    expected
}

fn text(path: &Path) -> &str {
    path.to_str().unwrap()
}

fn run(args: &[&str], input: Option<&str>, expected_exit: i32) -> Value {
    let mut child = Command::new(env!("CARGO_BIN_EXE_kitcontrol"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    if let Some(input) = input {
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
    } else {
        drop(child.stdin.take());
    }
    let output = child.wait_with_output().unwrap();
    assert_eq!(
        output.status.code(),
        Some(expected_exit),
        "args: {args:?}\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let response: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(response["api_version"], 1);
    assert_eq!(
        response.as_object().unwrap().len(),
        3,
        "exact versioned response envelope"
    );
    if response["ok"] == true {
        assert!(response.get("result").is_some());
        assert!(response.get("error").is_none());
        assert!(
            output.stderr.is_empty(),
            "successful requests must keep stdout machine readable and stderr clear"
        );
    } else {
        assert_eq!(response["ok"], false);
        assert!(response.get("result").is_none());
        assert_eq!(response["error"].as_object().unwrap().len(), 2);
        assert!(!output.stderr.is_empty());
    }
    response
}

fn request(request: Value, expected_exit: i32) -> Value {
    run(&["request", "-"], Some(&request.to_string()), expected_exit)
}

fn save(path: &Path, fixture: Value) {
    assert_eq!(
        request(
            json!({"action":"save","path":text(path),"document":fixture}),
            0
        ),
        json!({
            "api_version":1,"ok":true,"result":{"path":text(path),"files":4}
        })
    );
}

fn prepare_plan(root: &Path) -> (std::path::PathBuf, std::path::PathBuf, Value) {
    let workspace = root.join("desired");
    let device = root.join("device.json");
    let plan = root.join("reviewed-plan.json");
    save(&workspace, edited_document());
    fs::write(&device, snapshot().to_string()).unwrap();
    let response = run(
        &[
            "plan",
            "--baseline",
            text(&device),
            "--workspace",
            text(&workspace),
            "--output",
            text(&plan),
        ],
        None,
        0,
    );
    let persisted: Value = serde_json::from_slice(&fs::read(&plan).unwrap()).unwrap();
    assert_eq!(
        response,
        json!({"api_version":1,"ok":true,"result":persisted})
    );
    assert_eq!(persisted["format_version"], 1);
    assert_eq!(persisted["expected_device"], snapshot()["identity"]);
    assert_eq!(persisted["baseline"], document());
    assert_eq!(persisted["desired"], edited_document());
    assert_eq!(
        persisted["changes"],
        json!([
            {"path":"/kits/amber/name","before":"Amber test kit","after":"CLI edited amber"}
        ])
    );
    (device, plan, persisted)
}

#[test]
fn save_inspect_validate_and_offline_edit_use_the_public_cli_without_changing_device() {
    let root = tempfile::tempdir().unwrap();
    let workspace = root.path().join("original");
    let edited = root.path().join("edited");
    let device = root.path().join("device.json");
    let operation = root.path().join("operation.json");
    save(&workspace, document());
    fs::write(&device, snapshot().to_string()).unwrap();
    let device_before = fs::read(&device).unwrap();
    fs::write(
        &operation,
        r#"{"kind":"rename_kit","kit_id":"amber","name":"CLI edited amber"}"#,
    )
    .unwrap();
    assert_eq!(
        run(&["inspect", text(&workspace)], None, 0),
        json!({"api_version":1,"ok":true,"result":document()})
    );
    assert_eq!(
        run(&["validate", text(&workspace)], None, 0),
        json!({"api_version":1,"ok":true,"result":{"valid":true}})
    );
    assert_eq!(
        run(
            &[
                "edit",
                text(&workspace),
                "--operation",
                text(&operation),
                "--output",
                text(&edited)
            ],
            None,
            0
        ),
        json!({"api_version":1,"ok":true,"result":{"path":text(&edited),"files":4}})
    );
    assert_eq!(
        run(&["inspect", text(&edited)], None, 0),
        json!({"api_version":1,"ok":true,"result":edited_document()})
    );
    assert_eq!(
        run(&["inspect", text(&workspace)], None, 0),
        json!({"api_version":1,"ok":true,"result":document()})
    );
    assert_eq!(fs::read(&device).unwrap(), device_before);
    assert_eq!(
        run(&["diff", text(&workspace), text(&edited)], None, 0),
        json!({
            "api_version":1,"ok":true,"result":[{"path":"/kits/amber/name","before":"Amber test kit","after":"CLI edited amber"}]
        })
    );
}

#[test]
fn copy_and_reorder_commands_preserve_complete_kit_data_and_setlist_identity() {
    let root = tempfile::tempdir().unwrap();
    let workspace = root.path().join("original");
    let copied = root.path().join("copied");
    let reordered = root.path().join("reordered");
    save(&workspace, document());
    assert_eq!(
        run(
            &[
                "copy-kit",
                text(&workspace),
                "--source-kit",
                "amber",
                "--target-slot",
                "2",
                "--new-kit-id",
                "amber-copy",
                "--output",
                text(&copied)
            ],
            None,
            0
        ),
        json!({"api_version":1,"ok":true,"result":{"path":text(&copied),"files":5}})
    );
    let mut expected = document();
    expected["kits"]["amber-copy"] = document()["kits"]["amber"].clone();
    expected["slots"] = json!(["amber", "amber-copy"]);
    assert_eq!(
        run(&["inspect", text(&copied)], None, 0),
        json!({"api_version":1,"ok":true,"result":expected})
    );
    assert_eq!(
        run(
            &[
                "reorder",
                text(&workspace),
                "--order",
                "blue,amber",
                "--output",
                text(&reordered)
            ],
            None,
            0
        ),
        json!({"api_version":1,"ok":true,"result":{"path":text(&reordered),"files":4}})
    );
    let mut expected = document();
    expected["slots"] = json!(["blue", "amber"]);
    expected["setlists"][0]["slots"] = json!([2, 1, 2]);
    assert_eq!(
        run(&["inspect", text(&reordered)], None, 0),
        json!({"api_version":1,"ok":true,"result":expected})
    );
}

#[test]
fn copy_pad_command_preserves_layer_settings_and_target_kit_metadata() {
    let root = tempfile::tempdir().unwrap();
    let workspace = root.path().join("original");
    let copied = root.path().join("copied-pad");
    save(&workspace, document());
    assert_eq!(
        run(
            &[
                "copy-pad",
                text(&workspace),
                "--source-kit",
                "amber",
                "--source-pad",
                "snare.head",
                "--target-kit",
                "blue",
                "--target-pad",
                "kick.head",
                "--output",
                text(&copied),
            ],
            None,
            0,
        ),
        json!({"api_version":1,"ok":true,"result":{"path":text(&copied),"files":4}})
    );
    let mut expected = document();
    expected["kits"]["blue"]["pads"]["kick.head"] = json!({
        "instrument":"synth.snare.01", "tuning":-17, "muffling":3,
        "volume":72, "pan":-11, "sub_instrument":"synth.tom.01", "layer_level":29
    });
    assert_eq!(
        run(&["inspect", text(&copied)], None, 0),
        json!({"api_version":1,"ok":true,"result":expected})
    );
    assert_eq!(
        run(&["inspect", text(&workspace)], None, 0)["result"],
        document()
    );
}

#[test]
fn invalid_edit_returns_a_structured_failure_and_writes_no_destination() {
    let root = tempfile::tempdir().unwrap();
    let workspace = root.path().join("original");
    let edited = root.path().join("rejected");
    let operation = root.path().join("invalid-operation.json");
    save(&workspace, document());
    fs::write(&operation, r#"{"kind":"set_parameter","kit_id":"amber","pad":"snare.head","parameter":"volume","value":101}"#).unwrap();
    assert_eq!(
        run(
            &[
                "edit",
                text(&workspace),
                "--operation",
                text(&operation),
                "--output",
                text(&edited)
            ],
            None,
            2
        ),
        json!({
            "api_version":1,"ok":false,"error":{"code":"out_of_range","message":"volume must be between 0 and 100"}
        })
    );
    assert!(!edited.exists());
    assert_eq!(
        run(&["inspect", text(&workspace)], None, 0)["result"],
        document()
    );
}

#[test]
fn apply_success_disconnect_and_readback_mismatch_have_truthful_exit_codes_and_persistence() {
    for (fault, status, exit, written, verified, message, name) in [
        (
            "none",
            "verified",
            0,
            1,
            1,
            "Simulator document written and read back",
            "CLI edited amber",
        ),
        (
            "disconnect",
            "failed",
            4,
            0,
            0,
            "Injected simulator disconnect before document write",
            "Amber test kit",
        ),
        (
            "readback-mismatch",
            "partial",
            4,
            1,
            0,
            "Readback differs; device state is not verified. Read again before recovery.",
            "Injected mismatch",
        ),
    ] {
        let root = tempfile::tempdir().unwrap();
        let (device, plan, _) = prepare_plan(root.path());
        let original_device_bytes = fs::read(&device).unwrap();
        let mut expected_snapshot = snapshot();
        expected_snapshot["document"]["kits"]["amber"]["name"] = json!(name);
        let changes = if status == "partial" {
            json!([{"path":"/kits/amber/name","before":"Injected mismatch","after":"CLI edited amber"}])
        } else {
            json!([])
        };
        assert_eq!(
            run(
                &[
                    "apply",
                    "--device",
                    text(&device),
                    "--plan",
                    text(&plan),
                    "--fault",
                    fault
                ],
                None,
                exit
            ),
            json!({
                "api_version":1,"ok":true,"result":{
                    "status":status,"written":written,"verified":verified,"total":1,
                    "message":message,"snapshot":expected_snapshot,"changes":changes
                }
            })
        );
        let actual: Value = serde_json::from_slice(&fs::read(&device).unwrap()).unwrap();
        assert_eq!(actual, expected_snapshot);
        if status == "failed" {
            assert_eq!(fs::read(&device).unwrap(), original_device_bytes);
        }
    }
}

#[test]
fn stale_and_wrong_device_plans_fail_without_overwriting_device_state() {
    for wrong_device in [false, true] {
        let root = tempfile::tempdir().unwrap();
        let (device, plan, _) = prepare_plan(root.path());
        let mut changed = snapshot();
        if wrong_device {
            changed["identity"]["device_id"] = json!("other-test-device");
        } else {
            changed["document"]["kits"]["blue"]["name"] = json!("External edit");
        }
        fs::write(&device, changed.to_string()).unwrap();
        let before = fs::read(&device).unwrap();
        assert_eq!(
            run(
                &["apply", "--device", text(&device), "--plan", text(&plan)],
                None,
                4
            ),
            json!({
                "api_version":1,"ok":true,"result":{
                    "status":"conflict","written":0,"verified":0,"total":1,
                    "message":"Device identity or baseline changed; create a new plan",
                    "snapshot":changed,"changes":[]
                }
            })
        );
        assert_eq!(fs::read(&device).unwrap(), before);
    }
}

#[test]
fn forged_plan_fails_with_structured_error_and_no_device_write() {
    let root = tempfile::tempdir().unwrap();
    let (device, plan, mut forged) = prepare_plan(root.path());
    forged["changes"] = json!([]);
    fs::write(&plan, forged.to_string()).unwrap();
    let before = fs::read(&device).unwrap();
    assert_eq!(
        run(
            &["apply", "--device", text(&device), "--plan", text(&plan)],
            None,
            2
        ),
        json!({
            "api_version":1,"ok":false,"error":{"code":"invalid_plan","message":"Plan metadata or changes do not match its documents"}
        })
    );
    assert_eq!(fs::read(&device).unwrap(), before);
}

#[test]
fn reviewed_plan_cannot_be_silently_replaced() {
    let root = tempfile::tempdir().unwrap();
    let (device, plan, _) = prepare_plan(root.path());
    let before = fs::read(&plan).unwrap();
    assert_eq!(
        run(
            &[
                "plan",
                "--baseline",
                text(&device),
                "--workspace",
                text(&root.path().join("desired")),
                "--output",
                text(&plan)
            ],
            None,
            2
        ),
        json!({
            "api_version":1,"ok":false,"error":{"code":"destination_exists","message":"Destination exists; choose a new path"}
        })
    );
    assert_eq!(fs::read(&plan).unwrap(), before);
}

#[test]
fn verify_mismatch_uses_nonzero_exit_and_reports_the_actual_difference() {
    let root = tempfile::tempdir().unwrap();
    let (device, _, _) = prepare_plan(root.path());
    let response = run(
        &[
            "verify",
            "--device",
            text(&device),
            "--workspace",
            text(&root.path().join("desired")),
        ],
        None,
        5,
    );
    assert_eq!(response["ok"], true);
    assert_eq!(response["result"]["matches"], false);
    assert_eq!(
        response["result"]["changes"],
        json!([
            {"path":"/kits/amber/name","before":"Amber test kit","after":"CLI edited amber"}
        ])
    );
    assert_ne!(
        response["result"]["actual_hash"],
        response["result"]["expected_hash"]
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&fs::read(device).unwrap()).unwrap(),
        snapshot()
    );
}

#[test]
fn interrupted_save_is_reported_until_explicit_cli_recovery() {
    let root = tempfile::tempdir().unwrap();
    let workspace = root.path().join("workspace");
    save(&workspace, document());
    fs::write(
        workspace.join(".kitcontrol-pending.json"),
        edited_document().to_string(),
    )
    .unwrap();
    assert_eq!(
        run(&["inspect", text(&workspace)], None, 3),
        json!({
            "api_version":1,"ok":false,"error":{"code":"incomplete_save","message":"An interrupted save is pending. Run recover explicitly before loading."}
        })
    );
    assert_eq!(
        run(&["recover", text(&workspace)], None, 0),
        json!({
            "api_version":1,"ok":true,"result":{"path":text(&workspace),"files":4,"recovered":true}
        })
    );
    assert_eq!(
        run(&["inspect", text(&workspace)], None, 0),
        json!({"api_version":1,"ok":true,"result":edited_document()})
    );
}

#[test]
fn request_cli_enforces_explicit_api_versions_and_retains_legacy_omission() {
    for input in [
        json!({"action":"validate","document":document()}),
        json!({"action":"validate","api_version":1,"document":document()}),
    ] {
        assert_eq!(
            request(input, 0),
            json!({"api_version":1,"ok":true,"result":{"valid":true}})
        );
    }
    assert_eq!(
        request(
            json!({"action":"validate","api_version":2,"document":document()}),
            2
        ),
        json!({"api_version":1,"ok":false,"error":{
            "code":"unsupported_version","message":"Only request API version 1 is supported"
        }})
    );
}

#[test]
fn malformed_json_stdin_and_files_return_structured_nonzero_errors() {
    let response = run(&["request", "-"], Some("{"), 2);
    assert_eq!(response["ok"], false);
    assert_eq!(response["error"]["code"], "invalid_request");
    assert!(
        response["error"]["message"]
            .as_str()
            .unwrap()
            .contains("EOF")
    );
    let root = tempfile::tempdir().unwrap();
    let malformed = root.path().join("malformed.json");
    fs::write(&malformed, "{").unwrap();
    let response = run(&["request", text(&malformed)], None, 2);
    assert_eq!(response["ok"], false);
    assert_eq!(response["error"]["code"], "invalid_json");
    assert!(
        response["error"]["message"]
            .as_str()
            .unwrap()
            .contains("EOF")
    );
}

//! Public CLI tests use the TD-27 byte-protocol emulator, not document injection.
use serde_json::{Value, json};
use std::{
    fs,
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_kitcontrol"))
        .args(args)
        .output()
        .unwrap()
}
fn request(value: Value) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_kitcontrol"))
        .args(["request", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(value.to_string().as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}
fn result(output: &std::process::Output) -> Value {
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(
        value["ok"].as_bool().unwrap(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    value["result"].clone()
}
fn write(path: &Path, value: Value) {
    fs::write(path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
}
#[test]
fn given_td27_emulator_when_cli_reads_plans_applies_then_actual_bytes_read_back() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("target.json");
    let snapshot = dir.path().join("snapshot.json");
    let changes = dir.path().join("changes.json");
    let plan = dir.path().join("plan.json");
    write(&target, json!({"transport":"emulator"}));
    let read = run(&[
        "td27",
        "read",
        "--target",
        target.to_str().unwrap(),
        "--kit",
        "1",
        "--output",
        snapshot.to_str().unwrap(),
    ]);
    assert!(read.status.success());
    let baseline = result(&read);
    assert_eq!(baseline["parameters"]["kit.name"], "Example");
    assert_eq!(baseline["parameters"]["kit.tempo"], 120);
    assert_eq!(baseline["simulated"], true);
    write(
        &changes,
        json!({"kit.tempo":126,"pad.snare.compressor_enabled":true}),
    );
    let planned = run(&[
        "td27",
        "plan",
        "--baseline",
        snapshot.to_str().unwrap(),
        "--changes",
        changes.to_str().unwrap(),
        "--output",
        plan.to_str().unwrap(),
    ]);
    assert!(planned.status.success());
    let reviewed = result(&planned);
    assert_eq!(
        reviewed["writes"][0],
        json!({"key":"kit.tempo","address":[4,0,0,46],"before":[0,0,7,8],"after":[0,0,7,14]})
    );
    assert_eq!(
        reviewed["writes"][1],
        json!({"key":"pad.snare.compressor_enabled","address":[4,1,65,1],"before":[0],"after":[1]})
    );
    write(&target, json!({"transport":"emulator","state":baseline}));
    let applied = run(&[
        "td27",
        "apply",
        "--target",
        target.to_str().unwrap(),
        "--plan",
        plan.to_str().unwrap(),
    ]);
    assert!(applied.status.success());
    let report = result(&applied);
    assert_eq!(report["status"], "verified");
    assert_eq!(report["written"], 2);
    assert_eq!(report["verified"], 2);
    assert_eq!(report["snapshot"]["parameters"]["kit.tempo"], 126);
    assert_eq!(report["snapshot"]["blocks"][2]["data"][1], 1);
    let collision = run(&[
        "td27",
        "read",
        "--target",
        target.to_str().unwrap(),
        "--kit",
        "1",
        "--output",
        snapshot.to_str().unwrap(),
    ]);
    assert_eq!(collision.status.code(), Some(2));
    assert_eq!(
        serde_json::from_slice::<Value>(&collision.stdout).unwrap()["error"]["code"],
        "destination_exists"
    );
}
#[test]
fn given_td27_transport_fault_when_request_reads_then_error_is_structured_and_nonzero() {
    for (fault, code, exit) in [
        ("timeout", "midi_timeout", 3),
        ("bad_checksum", "invalid_midi", 2),
        ("wrong_device", "invalid_midi", 2),
        ("wrong_model", "invalid_midi", 2),
    ] {
        let output = request(
            json!({"api_version":1,"action":"td27_read","kit":1,"target":{"transport":"emulator","fault":fault}}),
        );
        assert_eq!(output.status.code(), Some(exit));
        let value: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(value["ok"], false);
        assert_eq!(value["error"]["code"], code);
        assert!(!output.stderr.is_empty());
    }
}
#[test]
fn given_td27_readback_failure_when_apply_then_cli_does_not_report_success() {
    let baseline = result(&request(
        json!({"action":"td27_read","kit":1,"target":{"transport":"emulator","fault":"fragmented"}}),
    ));
    let plan = result(&request(
        json!({"action":"td27_plan","baseline":baseline,"changes":{"kit.tempo":121}}),
    ));
    for fault in ["disconnect", "readback_mismatch"] {
        let output = request(
            json!({"action":"td27_apply","target":{"transport":"emulator","state":baseline,"fault":fault},"plan":plan}),
        );
        assert_eq!(output.status.code(), Some(4));
        let report = result(&output);
        assert_eq!(report["status"], "partial");
        assert_eq!(report["verified"], 0);
    }
}
#[test]
fn given_unknown_firmware_or_parameter_when_request_then_no_silent_profile_fallback() {
    let output = request(
        json!({"action":"td27_read","kit":1,"target":{"transport":"emulator","firmware":"2.10"}}),
    );
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).unwrap()["error"]["code"],
        "unsupported_profile"
    );
    let baseline = result(&request(
        json!({"action":"td27_read","kit":1,"target":{"transport":"emulator"}}),
    ));
    let output = request(
        json!({"action":"td27_plan","baseline":baseline,"changes":{"pad.snare.tuning":12}}),
    );
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).unwrap()["error"]["code"],
        "unsupported_parameter"
    );
}

#[test]
fn given_sandbox_plan_when_native_apply_requested_then_no_hardware_port_is_opened() {
    let baseline = result(&request(json!({
        "action":"td27_read","kit":1,"target":{"transport":"emulator"}
    })));
    let plan = result(&request(json!({
        "action":"td27_plan","baseline":baseline,"changes":{"kit.tempo":126}
    })));
    let output = request(json!({
        "action":"td27_apply","plan":plan,
        "target":{"transport":"native","input":"Never open this input","output":"Never open this output","device_id":16,"firmware":"2.00"}
    }));
    assert_eq!(output.status.code(), Some(2));
    let response: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(response["ok"], false);
    // The target guard executes before MIDI initialization/discovery/connect.
    assert_eq!(response["error"]["code"], "invalid_target");
}

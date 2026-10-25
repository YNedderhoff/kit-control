pub mod edit;
pub mod error;
pub mod midi;
pub mod model;
pub mod storage;
pub mod td27;
pub mod td27_connection;
pub mod td27_emulator;
pub mod transfer;

use crate::{
    edit::EditOperation,
    error::{Error, Result},
    model::{DeviceIdentity, Document, Snapshot},
    transfer::{ChangePlan, Fault, Simulator},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum Request {
    MidiPorts {},
    Td27Identify {
        target: td27_connection::Target,
    },
    Td27Read {
        target: td27_connection::Target,
        kit: u8,
    },
    Td27Plan {
        baseline: td27::Td27Snapshot,
        changes: std::collections::BTreeMap<String, Value>,
    },
    Td27Apply {
        target: td27_connection::Target,
        plan: Box<td27::ChangePlan>,
    },
    Demo {},
    Devices {},
    Validate {
        document: Document,
    },
    Edit {
        document: Document,
        operation: EditOperation,
    },
    Diff {
        before: Document,
        after: Document,
    },
    Plan {
        baseline: Snapshot,
        desired: Document,
    },
    SimulateApply {
        device: Snapshot,
        plan: Box<ChangePlan>,
        #[serde(default)]
        fault: Fault,
    },
    Verify {
        device: Snapshot,
        expected: Document,
    },
    Load {
        path: String,
    },
    Save {
        path: String,
        document: Document,
    },
    Recover {
        path: String,
    },
}

#[derive(Debug, Serialize)]
pub struct Response {
    pub api_version: u32,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Error>,
}

impl Response {
    pub fn exit_code(&self) -> i32 {
        if let Some(error) = &self.error {
            return match error.code.as_str() {
                "io_error" | "workspace_busy" | "incomplete_save" | "midi_error" | "midi_port"
                | "midi_timeout" | "midi_disconnected" | "midi_overflow" | "device_busy" => 3,
                "hardware_locked" => 6,
                _ => 2,
            };
        }
        if let Some(result) = &self.result {
            if result
                .get("status")
                .and_then(Value::as_str)
                .is_some_and(|s| s != "verified")
            {
                return 4;
            }
            if result.get("matches") == Some(&Value::Bool(false)) {
                return 5;
            }
        }
        0
    }
}

pub fn dispatch(request: Request) -> Response {
    match execute(request) {
        Ok(result) => Response {
            api_version: 1,
            ok: true,
            result: Some(result),
            error: None,
        },
        Err(error) => Response {
            api_version: 1,
            ok: false,
            result: None,
            error: Some(error),
        },
    }
}

pub fn dispatch_json(input: &str) -> Response {
    let parsed = (|| -> Result<Request> {
        if input.len() > 16 * 1024 * 1024 {
            return Err(Error::new("file_too_large", "Request exceeds 16 MiB"));
        }
        let mut value: Value = serde_json::from_str(input)?;
        if let Some(fields) = value.as_object_mut()
            && let Some(version) = fields.remove("api_version")
            && version.as_u64() != Some(1)
        {
            return Err(Error::new(
                "unsupported_version",
                "Only request API version 1 is supported",
            ));
        }
        serde_json::from_value(value)
            .map_err(|error| Error::new("invalid_request", error.to_string()))
    })();
    match parsed {
        Ok(request) => dispatch(request),
        Err(mut error) => {
            if error.code == "invalid_json" {
                error.code = "invalid_request".into();
            }
            Response {
                api_version: 1,
                ok: false,
                result: None,
                error: Some(error),
            }
        }
    }
}

fn execute(request: Request) -> Result<Value> {
    match request {
        Request::MidiPorts {} => Ok(serde_json::to_value(midi::ports()?)?),
        Request::Td27Identify { target } => {
            Ok(serde_json::to_value(td27_connection::identify(target)?)?)
        }
        Request::Td27Read { target, kit } => {
            Ok(serde_json::to_value(td27_connection::read(target, kit)?)?)
        }
        Request::Td27Plan { baseline, changes } => {
            Ok(serde_json::to_value(td27::plan(&baseline, &changes)?)?)
        }
        Request::Td27Apply { target, plan } => Ok(serde_json::to_value(td27_connection::apply(
            target, &plan,
        )?)?),
        Request::Demo {} => {
            let document = model::demo_document();
            let snapshot = Snapshot {
                identity: DeviceIdentity {
                    device_id: "simulator-1".into(),
                    profile_id: document.profile_id.clone(),
                    simulated: true,
                },
                document: document.clone(),
            };
            Ok(
                json!({"document":document,"snapshot":snapshot,"profile":model::synthetic_profile(),"hardware_status":"Synthetic workspace; use the TD-27 connection for native MIDI or the packet emulator"}),
            )
        }
        Request::Devices {} => Ok(
            json!({"devices":[{"device_id":"simulator-1","name":"Synthetic parameter simulator","simulated":true}],"hardware_status":"TD-27 native MIDI available; enumerate ports explicitly with midi_ports","opens_hardware_ports":false}),
        ),
        Request::Validate { document } => {
            document.validate()?;
            Ok(json!({"valid":true}))
        }
        Request::Edit {
            document,
            operation,
        } => Ok(serde_json::to_value(edit::edit(&document, operation)?)?),
        Request::Diff { before, after } => {
            Ok(serde_json::to_value(transfer::diff(&before, &after)?)?)
        }
        Request::Plan { baseline, desired } => {
            Ok(serde_json::to_value(transfer::plan(&baseline, &desired)?)?)
        }
        Request::SimulateApply {
            device,
            plan,
            fault,
        } => {
            let mut simulator = Simulator::new(device)?;
            Ok(serde_json::to_value(transfer::apply(
                &mut simulator,
                &plan,
                fault,
            )?)?)
        }
        Request::Verify { device, expected } => {
            device.validate()?;
            let changes = transfer::diff(&device.document, &expected)?;
            Ok(
                json!({"matches":changes.is_empty(),"actual_hash":transfer::fingerprint(&device.document)?,"expected_hash":transfer::fingerprint(&expected)?,"changes":changes}),
            )
        }
        Request::Load { path } => Ok(serde_json::to_value(storage::load(
            &storage::workspace_path(&path)?,
        )?)?),
        Request::Save { path, document } => {
            let files = storage::save(&storage::workspace_path(&path)?, &document)?;
            Ok(json!({"path":path,"files":files}))
        }
        Request::Recover { path } => {
            let files = storage::recover(&storage::workspace_path(&path)?)?;
            Ok(json!({"path":path,"files":files,"recovered":true}))
        }
    }
}

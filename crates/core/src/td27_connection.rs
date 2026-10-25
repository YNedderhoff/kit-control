//! Select transport explicitly. Both routes execute the identical TD-27 client.
use crate::{
    error::{Error, Result},
    midi::NativeMidi,
    td27::{ApplyReport, ChangePlan, Identity, Td27Client, Td27Snapshot},
    td27_emulator::{EmulatorFault, Td27Emulator},
};
use serde::Deserialize;
fn default_device() -> u8 {
    16
}
fn default_firmware() -> String {
    "2.00".into()
}
#[derive(Debug, Deserialize)]
#[serde(tag = "transport", rename_all = "snake_case", deny_unknown_fields)]
pub enum Target {
    Native {
        input: String,
        output: String,
        device_id: u8,
        firmware: String,
    },
    Emulator {
        #[serde(default)]
        state: Option<Td27Snapshot>,
        #[serde(default = "default_device")]
        device_id: u8,
        #[serde(default = "default_firmware")]
        firmware: String,
        #[serde(default)]
        fault: EmulatorFault,
    },
}
// The concrete MIDI connection lives on the calling worker thread, not in UI state.
macro_rules! with_client {
    ($target:expr,$kit:expr,$client:ident,$body:expr) => {
        match $target {
            Target::Native {
                input,
                output,
                device_id,
                firmware,
            } => {
                let mut $client =
                    Td27Client::new(NativeMidi::open(&input, &output)?, device_id, firmware)?;
                $body
            }
            Target::Emulator {
                state,
                device_id,
                firmware,
                fault,
            } => {
                let mut $client = Td27Client::new(
                    Td27Emulator::new(device_id, $kit, state.as_ref(), fault)?,
                    device_id,
                    firmware,
                )?;
                $body
            }
        }
    };
}
pub fn identify(target: Target) -> Result<Identity> {
    with_client!(target, 1, client, client.identify())
}
pub fn read(target: Target, kit: u8) -> Result<Td27Snapshot> {
    with_client!(target, kit, client, client.read_kit(kit))
}
pub fn apply(target: Target, plan: &ChangePlan) -> Result<ApplyReport> {
    // Reject tampering before even opening a MIDI port.
    plan.validate()?;
    if let Target::Native { input, output, .. } = &target {
        let expected = crate::td27::MidiEndpoint {
            input: input.clone(),
            output: output.clone(),
        };
        if plan.baseline.endpoint.as_ref() != Some(&expected) || plan.baseline.simulated {
            return Err(Error::new(
                "invalid_target",
                "Plan was not read from these native MIDI ports",
            ));
        }
    }
    with_client!(target, plan.baseline.kit, client, client.apply(plan))
}

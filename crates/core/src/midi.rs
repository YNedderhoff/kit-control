//! Desktop MIDI transport. Protocol validation stays in the TD-27 client.
use crate::{
    error::{Error, Result},
    td27::{MidiEndpoint, PacketIo},
};
use fs2::FileExt;
use midir::{Ignore, MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    fs::{File, OpenOptions},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver},
    },
    time::Duration,
};

fn midi_error(error: impl std::fmt::Display) -> Error {
    Error::new("midi_error", error.to_string())
}
#[derive(Debug, Serialize)]
pub struct Ports {
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
}
pub fn ports() -> Result<Ports> {
    let input = MidiInput::new("KitControl discovery").map_err(midi_error)?;
    let output = MidiOutput::new("KitControl discovery").map_err(midi_error)?;
    Ok(Ports {
        inputs: input
            .ports()
            .iter()
            .map(|p| input.port_name(p).map_err(midi_error))
            .collect::<Result<_>>()?,
        outputs: output
            .ports()
            .iter()
            .map(|p| output.port_name(p).map_err(midi_error))
            .collect::<Result<_>>()?,
    })
}
pub struct NativeMidi {
    endpoint: MidiEndpoint,
    _input: MidiInputConnection<()>,
    output: MidiOutputConnection,
    messages: Receiver<Vec<u8>>,
    overflow: Arc<AtomicBool>,
    _lock: File,
}
impl NativeMidi {
    /// Exact, unique names are required; choosing matching input/output is explicit.
    pub fn open(input_name: &str, output_name: &str) -> Result<Self> {
        let mut input = MidiInput::new("KitControl TD-27 input").map_err(midi_error)?;
        input.ignore(Ignore::None);
        let output = MidiOutput::new("KitControl TD-27 output").map_err(midi_error)?;
        let inputs = input
            .ports()
            .into_iter()
            .filter(|p| input.port_name(p).ok().as_deref() == Some(input_name))
            .collect::<Vec<_>>();
        let outputs = output
            .ports()
            .into_iter()
            .filter(|p| output.port_name(p).ok().as_deref() == Some(output_name))
            .collect::<Vec<_>>();
        if inputs.len() != 1 || outputs.len() != 1 {
            return Err(Error::new(
                "midi_port",
                "Select unique, currently present input and output port names",
            ));
        }
        // Lock by output, independent of the chosen input. GUI and CLI must not race.
        let lock_path = std::env::temp_dir().join(format!(
            "kitcontrol-midi-{:x}.lock",
            Sha256::digest(output_name.as_bytes())
        ));
        if std::fs::symlink_metadata(&lock_path).is_ok_and(|m| m.file_type().is_symlink()) {
            return Err(Error::new(
                "midi_error",
                "MIDI lock must not be a symbolic link",
            ));
        }
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(lock_path)?;
        lock.try_lock_exclusive().map_err(|_| {
            Error::new(
                "device_busy",
                "Another KitControl operation is using this MIDI output",
            )
        })?;
        let (sender, messages) = mpsc::sync_channel(256);
        let overflow = Arc::new(AtomicBool::new(false));
        let callback_overflow = overflow.clone();
        let connection = input
            .connect(
                &inputs[0],
                "KitControl TD-27 receive",
                move |_, bytes, _| {
                    if bytes.len() > 65536 || sender.try_send(bytes.to_vec()).is_err() {
                        callback_overflow.store(true, Ordering::Relaxed);
                    }
                },
                (),
            )
            .map_err(midi_error)?;
        let output = output
            .connect(&outputs[0], "KitControl TD-27 send")
            .map_err(midi_error)?;
        Ok(Self {
            endpoint: MidiEndpoint {
                input: input_name.into(),
                output: output_name.into(),
            },
            _input: connection,
            output,
            messages,
            overflow,
            _lock: lock,
        })
    }
}
impl PacketIo for NativeMidi {
    fn endpoint(&self) -> Option<MidiEndpoint> {
        Some(self.endpoint.clone())
    }
    fn send(&mut self, packet: &[u8]) -> Result<()> {
        self.check_input_health()?;
        self.output.send(packet).map_err(midi_error)
    }
    fn receive(&mut self, timeout: Duration) -> Result<Vec<u8>> {
        self.check_input_health()?;
        let packet = self
            .messages
            .recv_timeout(timeout)
            .map_err(|error| match error {
                mpsc::RecvTimeoutError::Timeout => {
                    Error::new("midi_timeout", "No response from selected TD-27 MIDI input")
                }
                _ => Error::new("midi_disconnected", "MIDI input disconnected"),
            })?;
        self.check_input_health()?;
        Ok(packet)
    }
}
impl NativeMidi {
    fn check_input_health(&self) -> Result<()> {
        if self.overflow.load(Ordering::Relaxed) {
            return Err(Error::new(
                "midi_overflow",
                "MIDI input queue overflowed; session stopped",
            ));
        }
        Ok(())
    }
}

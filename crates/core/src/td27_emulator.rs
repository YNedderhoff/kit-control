//! Parameter-only TD-27 emulator. Uses real request/reply envelopes, no audio.
use crate::{
    error::{Error, Result},
    td27::{self, MemoryBlock, PacketIo, Td27Snapshot},
};
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, time::Duration};
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EmulatorFault {
    #[default]
    None,
    Timeout,
    BadChecksum,
    WrongDevice,
    WrongModel,
    Disconnect,
    ReadbackMismatch,
    Fragmented,
}
pub struct Td27Emulator {
    device: u8,
    blocks: Vec<MemoryBlock>,
    queue: VecDeque<Vec<u8>>,
    fault: EmulatorFault,
}
fn number(bytes: &[u8]) -> usize {
    bytes.iter().fold(0, |n, b| n * 128 + usize::from(*b))
}
impl Td27Emulator {
    pub fn new(
        device: u8,
        kit: u8,
        state: Option<&Td27Snapshot>,
        fault: EmulatorFault,
    ) -> Result<Self> {
        td27::encode_rq1(device, [4, 0, 0, 0], [0, 0, 0, 1])?;
        if !(1..=100).contains(&kit) {
            return Err(Error::new("invalid_parameter", "Kit must be 1..100"));
        }
        let blocks = if let Some(state) = state {
            state.validate()?;
            if !state.simulated || state.kit != kit || state.identity.device_id != device {
                return Err(Error::new(
                    "invalid_snapshot",
                    "Emulator state must be simulated and match the selected kit/device",
                ));
            }
            state.blocks.clone()
        } else {
            let n = (kit - 1) as u32 * 32768 + 4 * 128 * 128 * 128;
            let base = [
                ((n >> 21) & 127) as u8,
                ((n >> 14) & 127) as u8,
                ((n >> 7) & 127) as u8,
                (n & 127) as u8,
            ];
            let mut common = vec![0; 52];
            common[..12].copy_from_slice(b"Example     ");
            common[46..50].copy_from_slice(&[0, 0, 7, 8]);
            let mut blocks = vec![MemoryBlock {
                address: base,
                data: common,
            }];
            for i in 0..12 {
                blocks.push(MemoryBlock {
                    address: [base[0], base[1] + 1, 64 + i, 0],
                    data: vec![0; 11],
                });
            }
            blocks
        };
        Ok(Self {
            device,
            blocks,
            queue: VecDeque::new(),
            fault,
        })
    }
    fn reply(&mut self, mut frame: Vec<u8>) {
        if matches!(self.fault, EmulatorFault::Timeout) {
            return;
        }
        if frame.get(1) == Some(&0x41) {
            match self.fault {
                EmulatorFault::BadChecksum => {
                    let last = frame.len() - 2;
                    frame[last] ^= 1;
                }
                EmulatorFault::WrongDevice => frame[2] = if self.device == 16 { 17 } else { 16 },
                EmulatorFault::WrongModel => frame[6] = 0x64,
                _ => {}
            }
        }
        if matches!(self.fault, EmulatorFault::Fragmented) {
            for chunk in frame.chunks(3) {
                self.queue.push_back(chunk.to_vec());
            }
        } else {
            self.queue.push_back(frame);
        }
    }
}
impl PacketIo for Td27Emulator {
    fn simulated(&self) -> bool {
        true
    }
    fn wait(&mut self, _duration: Duration) {}
    fn send(&mut self, packet: &[u8]) -> Result<()> {
        if packet == [0xf0, 0x7e, self.device, 6, 1, 0xf7] {
            self.reply(vec![
                0xf0,
                0x7e,
                self.device,
                6,
                2,
                0x41,
                0x63,
                3,
                0,
                0,
                0,
                0,
                0,
                2,
                0xf7,
            ]);
            return Ok(());
        }
        if packet.len() < 15 || packet[..7] != [0xf0, 0x41, self.device, 0, 0, 0, 0x63] {
            return Err(Error::new(
                "invalid_midi",
                "Emulator rejected MIDI envelope",
            ));
        }
        let command = packet[7];
        if command == 0x12 {
            let block = td27::decode_dt1(packet, self.device)?;
            if matches!(self.fault, EmulatorFault::Disconnect) {
                return Err(Error::new(
                    "midi_disconnected",
                    "Injected disconnect during write",
                ));
            }
            let start = number(&block.address);
            for stored in &mut self.blocks {
                let base = number(&stored.address);
                if start >= base && start + block.data.len() <= base + stored.data.len() {
                    if !matches!(self.fault, EmulatorFault::ReadbackMismatch) {
                        stored.data[start - base..start - base + block.data.len()]
                            .copy_from_slice(&block.data);
                    }
                    return Ok(());
                }
            }
            return Err(Error::new(
                "invalid_midi",
                "Emulator write outside parameter scope",
            ));
        }
        if command != 0x11 || packet.len() != 18 {
            return Err(Error::new("invalid_midi", "Invalid emulator RQ1"));
        }
        let sum: u32 = packet[8..17].iter().map(|b| u32::from(*b)).sum();
        if !sum.is_multiple_of(128) || packet[8..17].iter().any(|b| *b > 127) || packet[17] != 0xf7
        {
            return Err(Error::new(
                "invalid_midi",
                "Invalid RQ1 checksum or payload",
            ));
        }
        let address: [u8; 4] = packet[8..12].try_into().unwrap();
        let size = number(&packet[12..16]);
        let start = number(&address);
        for stored in &self.blocks {
            let base = number(&stored.address);
            if start >= base && size > 0 && start + size <= base + stored.data.len() {
                let reply = td27::encode_dt1(
                    self.device,
                    address,
                    &stored.data[start - base..start - base + size],
                )?;
                self.reply(reply);
                return Ok(());
            }
        }
        Err(Error::new(
            "invalid_midi",
            "Emulator read outside parameter scope",
        ))
    }
    fn receive(&mut self, _timeout: Duration) -> Result<Vec<u8>> {
        self.queue
            .pop_front()
            .ok_or_else(|| Error::new("midi_timeout", "Emulated TD-27 did not reply"))
    }
}

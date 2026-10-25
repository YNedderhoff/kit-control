//! Original implementation of the TD-27 v2.00 documented MIDI interface.
//! Source and implemented scope: docs/sources.md and docs/support.md.
use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, VecDeque},
    time::{Duration, Instant},
};

pub trait PacketIo {
    fn send(&mut self, packet: &[u8]) -> Result<()>;
    /// Return a complete MIDI message or a fragment; never wait beyond timeout.
    fn receive(&mut self, timeout: Duration) -> Result<Vec<u8>>;
    fn wait(&mut self, duration: Duration) {
        std::thread::sleep(duration);
    }
    fn endpoint(&self) -> Option<MidiEndpoint> {
        None
    }
    fn simulated(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MidiEndpoint {
    pub input: String,
    pub output: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Identity {
    pub device_id: u8,
    pub software_revision: [u8; 4],
    /// Entered by the user. Universal identity is not a full firmware version.
    pub firmware: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MemoryBlock {
    pub address: [u8; 4],
    pub data: Vec<u8>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Td27Snapshot {
    pub format_version: u32,
    pub identity: Identity,
    pub kit: u8,
    #[serde(default)]
    pub simulated: bool,
    #[serde(default)]
    pub endpoint: Option<MidiEndpoint>,
    pub blocks: Vec<MemoryBlock>,
    pub parameters: BTreeMap<String, Value>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParameterWrite {
    pub key: String,
    pub address: [u8; 4],
    pub before: Vec<u8>,
    pub after: Vec<u8>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChangePlan {
    pub format_version: u32,
    pub baseline: Td27Snapshot,
    pub baseline_hash: String,
    pub changes: BTreeMap<String, Value>,
    pub writes: Vec<ParameterWrite>,
}
#[derive(Debug, Serialize)]
pub struct ApplyReport {
    pub status: String,
    pub written: usize,
    pub verified: usize,
    pub total: usize,
    pub message: String,
    pub snapshot: Option<Td27Snapshot>,
}
fn invalid(message: impl Into<String>) -> Error {
    Error::new("invalid_midi", message)
}
fn device_valid(device: u8) -> Result<()> {
    if !(0x10..=0x1f).contains(&device) {
        return Err(invalid(
            "TD-27 device ID must be 16..31 (module display 17..32)",
        ));
    }
    Ok(())
}
fn address_value(bytes: [u8; 4]) -> Result<u32> {
    if bytes.iter().any(|b| *b > 127) {
        return Err(invalid("Address/size must contain four 7-bit bytes"));
    }
    Ok(bytes.into_iter().fold(0, |n, b| n * 128 + u32::from(b)))
}
fn address_bytes(mut value: u32) -> Result<[u8; 4]> {
    if value >= 1 << 28 {
        return Err(invalid("Address overflow"));
    }
    let mut bytes = [0; 4];
    for byte in bytes.iter_mut().rev() {
        *byte = (value & 127) as u8;
        value >>= 7;
    }
    Ok(bytes)
}
fn add_address(address: [u8; 4], offset: u32) -> Result<[u8; 4]> {
    address_bytes(
        address_value(address)?
            .checked_add(offset)
            .ok_or_else(|| invalid("Address overflow"))?,
    )
}
fn checksum(bytes: &[u8]) -> u8 {
    (128 - (bytes.iter().map(|b| u32::from(*b)).sum::<u32>() % 128)) as u8 & 127
}
fn encode(device: u8, command: u8, address: [u8; 4], data: &[u8]) -> Result<Vec<u8>> {
    device_valid(device)?;
    address_value(address)?;
    if data.is_empty() || data.len() > 256 || data.iter().any(|b| *b > 127) {
        return Err(invalid("Payload must contain 1..256 7-bit bytes"));
    }
    let mut frame = vec![0xf0, 0x41, device, 0, 0, 0, 0x63, command];
    frame.extend(address);
    frame.extend(data);
    frame.push(checksum(&frame[8..]));
    frame.push(0xf7);
    Ok(frame)
}
pub fn encode_rq1(device: u8, address: [u8; 4], size: [u8; 4]) -> Result<Vec<u8>> {
    if address_value(size)? == 0 {
        return Err(invalid("Read size must be nonzero"));
    }
    encode(device, 0x11, address, &size)
}
pub fn encode_dt1(device: u8, address: [u8; 4], data: &[u8]) -> Result<Vec<u8>> {
    encode(device, 0x12, address, data)
}
/// Validate the whole DT1 envelope before exposing any data to callers.
pub fn decode_dt1(frame: &[u8], device: u8) -> Result<MemoryBlock> {
    if frame.len() < 15 || frame.len() > 270 || frame[0] != 0xf0 || frame.last() != Some(&0xf7) {
        return Err(invalid("Invalid DT1 length or framing"));
    }
    if frame[1..8] != [0x41, device, 0, 0, 0, 0x63, 0x12] {
        return Err(invalid("Wrong DT1 manufacturer, device, model or command"));
    }
    if frame[2..frame.len() - 1].iter().any(|b| *b > 127) {
        return Err(invalid("Non-7-bit DT1 payload"));
    }
    if checksum(&frame[8..frame.len() - 1]) != 0 {
        return Err(invalid("DT1 checksum mismatch"));
    }
    Ok(MemoryBlock {
        address: frame[8..12].try_into().unwrap(),
        data: frame[12..frame.len() - 2].to_vec(),
    })
}
fn kit_address(kit: u8) -> Result<[u8; 4]> {
    if !(1..=100).contains(&kit) {
        return Err(Error::new("invalid_parameter", "Kit must be 1..100"));
    }
    add_address([4, 0, 0, 0], u32::from(kit - 1) * 32768)
}
fn block_layout(kit: u8) -> Result<Vec<([u8; 4], usize)>> {
    let base = kit_address(kit)?;
    let mut layout = vec![(base, 52)];
    for i in 0..12 {
        layout.push((add_address(base, 16384 + (64 + i) * 128)?, 11));
    }
    Ok(layout)
}
const PADS: [&str; 12] = [
    "kick", "snare", "tom1", "tom2", "tom3", "hihat", "crash1", "crash2", "ride", "aux1", "aux2",
    "aux3",
];
fn bool_value(value: u8) -> Result<Value> {
    if value > 1 {
        return Err(invalid("Boolean parameter outside 0..1"));
    }
    Ok(json!(value == 1))
}
fn decode_parameters(blocks: &[MemoryBlock]) -> Result<BTreeMap<String, Value>> {
    let common = &blocks[0].data;
    if common[..12].iter().any(|b| !(1..=126).contains(b)) {
        return Err(invalid("Kit name outside documented ASCII range 1..126"));
    }
    let name = String::from_utf8(common[..12].to_vec())
        .map_err(|_| invalid("Invalid kit name"))?
        .trim_end_matches(' ')
        .to_string();
    let nibbles = &common[46..50];
    if nibbles.iter().any(|b| *b > 15) {
        return Err(invalid("Kit tempo is not four nibbles"));
    }
    let tempo = nibbles.iter().fold(0u32, |n, b| n * 16 + u32::from(*b));
    if !(20..=260).contains(&tempo) {
        return Err(invalid("Kit tempo outside 20..260"));
    }
    let mut params = BTreeMap::from([
        ("kit.name".into(), json!(name)),
        ("kit.tempo".into(), json!(tempo)),
        ("kit.tempo_enabled".into(), bool_value(common[45])?),
        ("kit.brush_enabled".into(), bool_value(common[42])?),
        ("kit.xstick_enabled".into(), bool_value(common[36])?),
    ]);
    for (index, pad) in PADS.iter().enumerate() {
        params.insert(
            format!("pad.{pad}.compressor_enabled"),
            bool_value(blocks[index + 1].data[1])?,
        );
    }
    Ok(params)
}
impl Td27Snapshot {
    pub fn validate(&self) -> Result<()> {
        device_valid(self.identity.device_id)?;
        if self.format_version != 1
            || self.identity.firmware != "2.00"
            || self.identity.software_revision.iter().any(|b| *b > 127)
        {
            return Err(Error::new(
                "unsupported_profile",
                "Only declared TD-27 firmware 2.00 and snapshot version 1 are supported",
            ));
        }
        let layout = block_layout(self.kit)?;
        if self.blocks.len() != layout.len() {
            return Err(invalid("Incomplete snapshot scope"));
        }
        for (block, (address, size)) in self.blocks.iter().zip(layout) {
            if block.address != address
                || block.data.len() != size
                || block.data.iter().any(|b| *b > 127)
            {
                return Err(invalid("Snapshot block does not match TD-27 scope"));
            }
        }
        if decode_parameters(&self.blocks)? != self.parameters {
            return Err(invalid("Snapshot parameters disagree with its bytes"));
        }
        Ok(())
    }
}
fn field(key: &str) -> Result<(usize, usize, usize)> {
    match key {
        "kit.name" => Ok((0, 0, 12)),
        "kit.tempo" => Ok((0, 46, 4)),
        "kit.tempo_enabled" => Ok((0, 45, 1)),
        "kit.brush_enabled" => Ok((0, 42, 1)),
        "kit.xstick_enabled" => Ok((0, 36, 1)),
        _ => {
            for (index, pad) in PADS.iter().enumerate() {
                if key == format!("pad.{pad}.compressor_enabled") {
                    return Ok((index + 1, 1, 1));
                }
            }
            Err(Error::new(
                "unsupported_parameter",
                format!("Unsupported TD-27 parameter: {key}"),
            ))
        }
    }
}
fn encode_parameter(key: &str, value: &Value) -> Result<Vec<u8>> {
    let bad = || Error::new("invalid_parameter", format!("Invalid value for {key}"));
    match key {
        "kit.name" => {
            let name = value.as_str().ok_or_else(bad)?;
            if name.is_empty() || name.len() > 12 || !name.bytes().all(|b| (32..=126).contains(&b))
            {
                return Err(bad());
            }
            let mut bytes = name.as_bytes().to_vec();
            bytes.resize(12, b' ');
            Ok(bytes)
        }
        "kit.tempo" => {
            let n = value
                .as_u64()
                .filter(|n| (20..=260).contains(n))
                .ok_or_else(bad)?;
            Ok(vec![
                ((n >> 12) & 15) as u8,
                ((n >> 8) & 15) as u8,
                ((n >> 4) & 15) as u8,
                (n & 15) as u8,
            ])
        }
        _ => {
            field(key)?;
            Ok(vec![u8::from(value.as_bool().ok_or_else(bad)?)])
        }
    }
}
pub fn plan(baseline: &Td27Snapshot, changes: &BTreeMap<String, Value>) -> Result<ChangePlan> {
    baseline.validate()?;
    let mut writes = vec![];
    for (key, value) in changes {
        let (index, offset, size) = field(key)?;
        let after = encode_parameter(key, value)?;
        let block = &baseline.blocks[index];
        let before = block.data[offset..offset + size].to_vec();
        if before != after {
            writes.push(ParameterWrite {
                key: key.clone(),
                address: add_address(block.address, offset as u32)?,
                before,
                after,
            });
        }
    }
    Ok(ChangePlan {
        format_version: 1,
        baseline: baseline.clone(),
        baseline_hash: format!("{:x}", Sha256::digest(serde_json::to_vec(baseline)?)),
        changes: changes.clone(),
        writes,
    })
}
impl ChangePlan {
    pub fn validate(&self) -> Result<()> {
        if self.format_version != 1 || plan(&self.baseline, &self.changes)? != *self {
            return Err(Error::new(
                "invalid_plan",
                "TD-27 plan does not match independently recomputed writes",
            ));
        }
        Ok(())
    }
}

pub struct Td27Client<T: PacketIo> {
    io: T,
    device_id: u8,
    firmware: String,
    pending: Vec<u8>,
    frames: VecDeque<Vec<u8>>,
}
impl<T: PacketIo> Td27Client<T> {
    pub fn new(io: T, device_id: u8, firmware: String) -> Result<Self> {
        device_valid(device_id)?;
        Ok(Self {
            io,
            device_id,
            firmware,
            pending: vec![],
            frames: VecDeque::new(),
        })
    }
    pub fn into_inner(self) -> T {
        self.io
    }
    fn frame(&mut self, deadline: Instant) -> Result<Vec<u8>> {
        let mut received = 0usize;
        loop {
            if let Some(frame) = self.frames.pop_front() {
                return Ok(frame);
            }
            let remaining = deadline
                .checked_duration_since(Instant::now())
                .ok_or_else(|| Error::new("midi_timeout", "Timed out waiting for TD-27 SysEx"))?;
            let chunk = self.io.receive(remaining)?;
            received += chunk.len();
            if received > 65536 {
                return Err(invalid("Too much unrelated MIDI data"));
            }
            if chunk.is_empty() {
                return Err(Error::new("midi_timeout", "No TD-27 response"));
            }
            for byte in chunk {
                if byte >= 0xf8 {
                    continue;
                } // MIDI realtime may occur inside SysEx.
                match byte {
                    0xf0 => {
                        if !self.pending.is_empty() {
                            return Err(invalid("Nested SysEx start"));
                        }
                        self.pending.push(byte);
                    }
                    0xf7 if !self.pending.is_empty() => {
                        self.pending.push(byte);
                        self.frames.push_back(std::mem::take(&mut self.pending));
                    }
                    0x80..=0xf7 if !self.pending.is_empty() => {
                        return Err(invalid("MIDI status inside SysEx"));
                    }
                    _ if !self.pending.is_empty() => self.pending.push(byte),
                    _ => {}
                }
                if self.pending.len() > 270 || self.frames.len() > 64 {
                    return Err(invalid("Oversized SysEx response"));
                }
            }
        }
    }
    pub fn identify(&mut self) -> Result<Identity> {
        self.io.send(&[0xf0, 0x7e, self.device_id, 6, 1, 0xf7])?;
        let frame = self.frame(Instant::now() + Duration::from_secs(2))?;
        if frame.len() != 15
            || frame[..10] != [0xf0, 0x7e, self.device_id, 6, 2, 0x41, 0x63, 3, 0, 0]
            || frame[14] != 0xf7
            || frame[10..14].iter().any(|b| *b > 127)
        {
            return Err(invalid(
                "Identity response is not the selected Roland TD-27",
            ));
        }
        Ok(Identity {
            device_id: self.device_id,
            software_revision: frame[10..14].try_into().unwrap(),
            firmware: self.firmware.clone(),
        })
    }
    fn read_block(&mut self, address: [u8; 4], size: usize) -> Result<MemoryBlock> {
        self.io.send(&encode_rq1(
            self.device_id,
            address,
            address_bytes(size as u32)?,
        )?)?;
        let start = address_value(address)?;
        let mut data = vec![None; size];
        let mut count = 0;
        let deadline = Instant::now() + Duration::from_secs(2);
        while count < size {
            let frame = self.frame(deadline)?;
            let block = decode_dt1(&frame, self.device_id)?;
            let offset = address_value(block.address)?
                .checked_sub(start)
                .ok_or_else(|| invalid("Response starts before requested range"))?
                as usize;
            if offset + block.data.len() > size {
                return Err(invalid("Response extends outside requested range"));
            }
            for (index, value) in block.data.into_iter().enumerate() {
                if data[offset + index].replace(value).is_some() {
                    return Err(invalid("Overlapping or repeated response bytes"));
                }
                count += 1;
            }
        }
        Ok(MemoryBlock {
            address,
            data: data.into_iter().map(Option::unwrap).collect(),
        })
    }
    pub fn read_kit(&mut self, kit: u8) -> Result<Td27Snapshot> {
        if self.firmware != "2.00" {
            return Err(Error::new(
                "unsupported_profile",
                "Enter firmware 2.00 from the module; other firmware profiles require review",
            ));
        }
        let layout = block_layout(kit)?;
        let identity = self.identify()?;
        let mut blocks = vec![];
        for (address, size) in layout {
            blocks.push(self.read_block(address, size)?);
            self.io.wait(Duration::from_millis(20));
        }
        let parameters = decode_parameters(&blocks)?;
        Ok(Td27Snapshot {
            format_version: 1,
            identity,
            kit,
            simulated: self.io.simulated(),
            endpoint: self.io.endpoint(),
            blocks,
            parameters,
        })
    }
    pub fn apply(&mut self, plan: &ChangePlan) -> Result<ApplyReport> {
        plan.validate()?;
        let total = plan.writes.len();
        let mut report = ApplyReport {
            status: "conflict".into(),
            written: 0,
            verified: 0,
            total,
            message: String::new(),
            snapshot: None,
        };
        if plan.baseline.identity.device_id != self.device_id
            || plan.baseline.identity.firmware != self.firmware
            || plan.baseline.simulated != self.io.simulated()
            || plan.baseline.endpoint != self.io.endpoint()
        {
            report.message =
                "Plan targets a different MIDI endpoint, connection type, device ID or firmware"
                    .into();
            return Ok(report);
        }
        let actual = self.read_kit(plan.baseline.kit)?;
        if actual != plan.baseline {
            report.message =
                "TD-27 changed since the baseline was read; read again and make a new plan".into();
            report.snapshot = Some(actual);
            return Ok(report);
        }
        for write in &plan.writes {
            if let Err(error) =
                self.io
                    .send(&encode_dt1(self.device_id, write.address, &write.after)?)
            {
                report.status = "partial".into();
                report.message = format!(
                    "Write may have reached the module: {}. Read again before recovery",
                    error.message
                );
                return Ok(report);
            }
            report.written += 1;
            self.io.wait(Duration::from_millis(20));
            let (block_index, offset, size) = field(&write.key)?;
            let block = &plan.baseline.blocks[block_index];
            match self.read_block(block.address, block.data.len()) {
                Ok(actual) if actual.data[offset..offset + size] == write.after => {
                    report.verified += 1
                }
                Ok(_) => {
                    report.status = "partial".into();
                    report.message = format!(
                        "Readback differs for {}; remaining writes stopped",
                        write.key
                    );
                    return Ok(report);
                }
                Err(error) => {
                    report.status = "partial".into();
                    report.message = format!(
                        "Readback failed: {}; remaining writes stopped",
                        error.message
                    );
                    return Ok(report);
                }
            }
            self.io.wait(Duration::from_millis(20));
        }
        match self.read_kit(plan.baseline.kit) {
            Ok(snapshot) => {
                let mut expected = plan.baseline.clone();
                for write in &plan.writes {
                    let (index, offset, size) = field(&write.key)?;
                    expected.blocks[index].data[offset..offset + size]
                        .copy_from_slice(&write.after);
                }
                expected.parameters = decode_parameters(&expected.blocks)?;
                report.verified = plan
                    .writes
                    .iter()
                    .filter(|write| {
                        let (index, offset, size) = field(&write.key).expect("validated parameter");
                        snapshot.blocks[index].data[offset..offset + size] == write.after
                    })
                    .count();
                report.status = if snapshot == expected {
                    "verified"
                } else {
                    "partial"
                }
                .into();
                report.message = if snapshot == expected {
                    "All planned parameters read back; this is not a complete kit or sample backup"
                } else {
                    "Final readback differs; read again before further changes"
                }
                .into();
                report.snapshot = Some(snapshot);
            }
            Err(error) => {
                report.status = "partial".into();
                report.verified = 0;
                report.message = format!(
                    "Final read failed: {}; final state is unverified",
                    error.message
                );
            }
        }
        Ok(report)
    }
}

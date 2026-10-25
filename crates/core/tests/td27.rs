//! Independent packet tests against the TD-27 MIDI Implementation v2.00,
//! 11 August 2022, pp. 5–6, 10–14, and 21 (KitPad definition):
//! https://static.roland.com/assets/media/pdf/TD-27_MIDI_Imple_eng04_W.pdf
//! Interface facts are cited; all parameter contents below are original test data.

use kitcontrol_core::{
    error::{Error, Result},
    td27::{self, Identity, MemoryBlock, MidiEndpoint, PacketIo, Td27Client, Td27Snapshot},
    td27_connection::{self, Target},
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, VecDeque},
    time::Duration,
};

#[derive(Clone, Copy, Default)]
enum ReplyMode {
    #[default]
    Complete,
    SplitMessages,
    ReverseMessages,
    ByteFragments,
    BadChecksum,
    WrongDevice,
    WrongModel,
    WrongAddress,
    NonSevenBit,
    MissingTail,
    Overlap,
}

// The mock consumes and produces wire messages. It does not call production
// encoders, decoders, address helpers, constants, or document simulators.
struct Module {
    device: u8,
    revision: [u8; 4],
    memory: BTreeMap<u32, u8>,
    incoming: VecDeque<Vec<u8>>,
    sent: Vec<Vec<u8>>,
    waits: Vec<Duration>,
    timeouts: Vec<Duration>,
    mode: ReplyMode,
    identity_reply: Option<Vec<u8>>,
    write_attempts: usize,
    fail_write: Option<usize>,
    mismatch: bool,
    fail_readback: bool,
    endpoint: Option<MidiEndpoint>,
    fail_final_read: bool,
    final_mismatch: bool,
}

fn number(address: &[u8]) -> u32 {
    address
        .iter()
        .enumerate()
        .map(|(i, byte)| u32::from(*byte) * 128_u32.pow((3 - i) as u32))
        .sum()
}

fn address(value: u32) -> [u8; 4] {
    [
        (value / 2_097_152 % 128) as u8,
        (value / 16_384 % 128) as u8,
        (value / 128 % 128) as u8,
        (value % 128) as u8,
    ]
}

fn reply(device: u8, start: u32, data: &[u8]) -> Vec<u8> {
    let mut result = vec![0xf0, 0x41, device, 0, 0, 0, 0x63, 0x12];
    result.extend(address(start));
    result.extend_from_slice(data);
    let sum: usize = result[8..].iter().map(|byte| usize::from(*byte)).sum();
    result.push(((128 - sum % 128) % 128) as u8);
    result.push(0xf7);
    result
}

fn common_bytes() -> Vec<u8> {
    let mut data = vec![0; 52];
    data[..12].copy_from_slice(b"TEST KIT A  ");
    data[12..28].copy_from_slice(b"ORIGINAL FIXTURE");
    data[0x24] = 1;
    data[0x2a] = 0;
    data[0x2d] = 1;
    data[0x2e..0x32].copy_from_slice(&[0, 0, 7, 11]);
    data[0x33] = 2;
    data
}

impl Module {
    fn new(base: [u8; 4]) -> Self {
        let base = number(&base);
        let mut memory = BTreeMap::new();
        for (offset, byte) in common_bytes().into_iter().enumerate() {
            memory.insert(base + offset as u32, byte);
        }
        for pad in 0..12_u32 {
            let bytes = [0, (pad % 2) as u8, 0, 0, 0, 0, 0, 0, 0, 0, 5];
            for (offset, byte) in bytes.into_iter().enumerate() {
                memory.insert(base + 16_384 + (64 + pad) * 128 + offset as u32, byte);
            }
        }
        Self {
            device: 0x10,
            revision: [0, 0, 0, 2],
            memory,
            incoming: VecDeque::new(),
            sent: vec![],
            waits: vec![],
            timeouts: vec![],
            mode: ReplyMode::Complete,
            identity_reply: None,
            write_attempts: 0,
            fail_write: None,
            mismatch: false,
            fail_readback: false,
            endpoint: None,
            fail_final_read: false,
            final_mismatch: false,
        }
    }

    fn client(self) -> Td27Client<Self> {
        let device = self.device;
        Td27Client::new(self, device, "2.00".into()).unwrap()
    }

    fn queue(&mut self, frames: Vec<Vec<u8>>) {
        if matches!(self.mode, ReplyMode::ByteFragments) {
            let mut bytes = Vec::new();
            for frame in frames {
                for byte in frame {
                    bytes.push(byte);
                    bytes.push(0xf8); // Realtime MIDI is allowed within SysEx.
                }
            }
            for chunk in bytes.chunks(3) {
                self.incoming.push_back(chunk.to_vec());
            }
        } else {
            self.incoming.extend(frames);
        }
    }
}

impl PacketIo for Module {
    fn send(&mut self, packet: &[u8]) -> Result<()> {
        self.sent.push(packet.to_vec());
        if packet == [0xf0, 0x7e, self.device, 6, 1, 0xf7] {
            if self.write_attempts > 0 && self.fail_final_read {
                return Err(Error::new(
                    "test_disconnect",
                    "test transport disconnected before final read",
                ));
            }
            if self.write_attempts > 0 && self.final_mismatch {
                self.memory.insert(number(&[4, 0, 0, 0x2d]), 1);
            }
            let mut identity = vec![0xf0, 0x7e, self.device, 6, 2, 0x41, 0x63, 3, 0, 0];
            identity.extend(self.revision);
            identity.push(0xf7);
            self.queue(vec![self.identity_reply.clone().unwrap_or(identity)]);
            return Ok(());
        }
        assert!(packet.len() >= 15, "short outbound packet: {packet:02x?}");
        assert_eq!(&packet[..7], &[0xf0, 0x41, self.device, 0, 0, 0, 0x63]);
        assert_eq!(packet.last(), Some(&0xf7));
        assert!(packet[1..packet.len() - 1].iter().all(|byte| *byte < 128));
        assert_eq!(
            packet[8..packet.len() - 1]
                .iter()
                .map(|byte| usize::from(*byte))
                .sum::<usize>()
                % 128,
            0
        );
        let start = number(&packet[8..12]);
        match packet[7] {
            0x11 => {
                assert_eq!(packet.len(), 18);
                let size = number(&packet[12..16]) as usize;
                assert!(size > 0);
                let data: Vec<_> = (0..size)
                    .map(|offset| {
                        *self
                            .memory
                            .get(&(start + offset as u32))
                            .expect("request outside independent mock memory")
                    })
                    .collect();
                let mut frames = match self.mode {
                    ReplyMode::SplitMessages
                    | ReplyMode::ReverseMessages
                    | ReplyMode::ByteFragments => data
                        .chunks(7)
                        .enumerate()
                        .map(|(i, chunk)| reply(self.device, start + i as u32 * 7, chunk))
                        .collect::<Vec<_>>(),
                    ReplyMode::MissingTail => vec![reply(self.device, start, &data[..size - 1])],
                    ReplyMode::Overlap => vec![
                        reply(self.device, start, &data[..2]),
                        reply(self.device, start + 1, &data[1..]),
                    ],
                    _ => vec![reply(self.device, start, &data)],
                };
                match self.mode {
                    ReplyMode::ReverseMessages => frames.reverse(),
                    ReplyMode::BadChecksum => {
                        let end = frames[0].len() - 2;
                        frames[0][end] ^= 1;
                    }
                    ReplyMode::WrongDevice => frames[0][2] = 0x11,
                    ReplyMode::WrongModel => frames[0][6] = 0x62,
                    ReplyMode::WrongAddress => frames[0] = reply(self.device, start + 4096, &data),
                    ReplyMode::NonSevenBit => frames[0][12] = 0x80,
                    _ => {}
                }
                self.queue(frames);
            }
            0x12 => {
                self.write_attempts += 1;
                if self.fail_write == Some(self.write_attempts) {
                    return Err(Error::new(
                        "test_disconnect",
                        "test transport disconnected during send",
                    ));
                }
                for (offset, byte) in packet[12..packet.len() - 2].iter().enumerate() {
                    *self
                        .memory
                        .get_mut(&(start + offset as u32))
                        .expect("write outside independent mock memory") = *byte;
                }
                if self.mismatch {
                    *self.memory.get_mut(&start).unwrap() ^= 1;
                }
            }
            other => panic!("unexpected command {other:02x}"),
        }
        Ok(())
    }

    fn receive(&mut self, timeout: Duration) -> Result<Vec<u8>> {
        self.timeouts.push(timeout);
        if self.fail_readback && self.write_attempts > 0 {
            return Err(Error::new(
                "test_disconnect",
                "test transport disconnected before readback",
            ));
        }
        self.incoming
            .pop_front()
            .ok_or_else(|| Error::new("midi_timeout", "test response exhausted"))
    }

    fn wait(&mut self, duration: Duration) {
        self.waits.push(duration);
    }

    fn endpoint(&self) -> Option<MidiEndpoint> {
        self.endpoint.clone()
    }
}

fn baseline() -> Td27Snapshot {
    let mut blocks = vec![MemoryBlock {
        address: [4, 0, 0, 0],
        data: common_bytes(),
    }];
    for (pad, enabled) in [
        (0x40, 0),
        (0x41, 1),
        (0x42, 0),
        (0x43, 1),
        (0x44, 0),
        (0x45, 1),
        (0x46, 0),
        (0x47, 1),
        (0x48, 0),
        (0x49, 1),
        (0x4a, 0),
        (0x4b, 1),
    ] {
        blocks.push(MemoryBlock {
            address: [4, 1, pad, 0],
            data: vec![0, enabled, 0, 0, 0, 0, 0, 0, 0, 0, 5],
        });
    }
    let parameters = serde_json::from_value(json!({
        "kit.name":"TEST KIT A", "kit.tempo":123, "kit.tempo_enabled":true,
        "kit.brush_enabled":false, "kit.xstick_enabled":true,
        "pad.kick.compressor_enabled":false, "pad.snare.compressor_enabled":true,
        "pad.tom1.compressor_enabled":false, "pad.tom2.compressor_enabled":true,
        "pad.tom3.compressor_enabled":false, "pad.hihat.compressor_enabled":true,
        "pad.crash1.compressor_enabled":false, "pad.crash2.compressor_enabled":true,
        "pad.ride.compressor_enabled":false, "pad.aux1.compressor_enabled":true,
        "pad.aux2.compressor_enabled":false, "pad.aux3.compressor_enabled":true
    }))
    .unwrap();
    Td27Snapshot {
        format_version: 1,
        identity: Identity {
            device_id: 0x10,
            software_revision: [0, 0, 0, 2],
            firmware: "2.00".into(),
        },
        kit: 1,
        simulated: false,
        endpoint: None,
        blocks,
        parameters,
    }
}

fn change(key: &str, value: Value) -> BTreeMap<String, Value> {
    BTreeMap::from([(key.into(), value)])
}

#[test]
fn rq1_matches_an_independent_literal_packet() {
    assert_eq!(
        td27::encode_rq1(0x10, [4, 1, 0x41, 1], [0, 0, 0, 1]).unwrap(),
        [
            0xf0, 0x41, 0x10, 0, 0, 0, 0x63, 0x11, 4, 1, 0x41, 1, 0, 0, 0, 1, 0x38, 0xf7
        ]
    );
}

#[test]
fn dt1_matches_an_independent_literal_packet_and_decodes_its_payload() {
    let golden = [
        0xf0, 0x41, 0x10, 0, 0, 0, 0x63, 0x12, 4, 1, 1, 1, 0, 5, 0x74, 0xf7,
    ];
    assert_eq!(
        td27::encode_dt1(0x10, [4, 1, 1, 1], &[0, 5]).unwrap(),
        golden
    );
    let decoded = td27::decode_dt1(&golden, 0x10).unwrap();
    assert_eq!(decoded.address, [4, 1, 1, 1]);
    assert_eq!(decoded.data, [0, 5]);
}

#[test]
fn packet_encoders_reject_invalid_device_address_size_and_data() {
    for device in [0, 0x0f, 0x20, 0x7f, 0x80] {
        assert!(td27::encode_dt1(device, [4, 0, 0, 0], &[1]).is_err());
    }
    assert!(td27::encode_dt1(0x10, [4, 0, 0x80, 0], &[1]).is_err());
    for data in [vec![], vec![0x80], vec![1; 257]] {
        assert!(td27::encode_dt1(0x10, [4, 0, 0, 0], &data).is_err());
    }
    assert!(td27::encode_rq1(0x10, [4, 0, 0, 0], [0; 4]).is_err());
    assert!(td27::encode_rq1(0x10, [4, 0, 0, 0], [0, 0, 0, 0x80]).is_err());
}

#[test]
fn identity_request_is_directed_and_reply_keeps_revision_separate_from_firmware() {
    let mut client = Module::new([4, 0, 0, 0]).client();
    let identity = client.identify().unwrap();
    assert_eq!(identity.device_id, 0x10);
    assert_eq!(identity.software_revision, [0, 0, 0, 2]);
    assert_eq!(identity.firmware, "2.00");
    assert_eq!(
        client.into_inner().sent,
        vec![vec![0xf0, 0x7e, 0x10, 6, 1, 0xf7]]
    );
}

#[test]
fn identity_rejects_wrong_device_manufacturer_family_and_invalid_revision() {
    for (offset, byte) in [(2, 0x11), (5, 0x42), (6, 0x62), (8, 1), (10, 0x80)] {
        let mut io = Module::new([4, 0, 0, 0]);
        let mut identity = vec![
            0xf0, 0x7e, 0x10, 6, 2, 0x41, 0x63, 3, 0, 0, 0, 0, 0, 2, 0xf7,
        ];
        identity[offset] = byte;
        io.identity_reply = Some(identity);
        assert!(io.client().identify().is_err(), "identity offset {offset}");
    }
}

#[test]
fn read_kit_requests_only_the_declared_scope_and_decodes_independent_parameters() {
    let mut client = Module::new([4, 0, 0, 0]).client();
    let snapshot = client.read_kit(1).unwrap();
    assert_eq!(snapshot.kit, 1);
    assert_eq!(snapshot.blocks.len(), 13);
    assert!(!snapshot.simulated);
    assert_eq!(snapshot.blocks[0].data, common_bytes());
    assert_eq!(snapshot.parameters["kit.name"], "TEST KIT A");
    assert_eq!(snapshot.parameters["kit.tempo"], 123);
    assert_eq!(snapshot.parameters["kit.tempo_enabled"], true);
    assert_eq!(snapshot.parameters["kit.brush_enabled"], false);
    assert_eq!(snapshot.parameters["kit.xstick_enabled"], true);
    assert_eq!(snapshot.parameters["pad.kick.compressor_enabled"], false);
    assert_eq!(snapshot.parameters["pad.snare.compressor_enabled"], true);
    assert_eq!(snapshot.parameters["pad.aux3.compressor_enabled"], true);
    let io = client.into_inner();
    let reads: Vec<_> = io
        .sent
        .iter()
        .filter(|frame| frame.get(7) == Some(&0x11))
        .collect();
    assert_eq!(reads.len(), 13);
    assert_eq!(&reads[0][8..16], &[4, 0, 0, 0, 0, 0, 0, 52]);
    for (request, pad) in reads[1..].iter().zip(0x40..=0x4b) {
        assert_eq!(&request[8..16], &[4, 1, pad, 0, 0, 0, 0, 11]);
    }
    assert_eq!(io.write_attempts, 0);
    assert!(
        io.timeouts
            .iter()
            .all(|duration| *duration > Duration::ZERO && *duration <= Duration::from_secs(2))
    );
}

#[test]
fn kit_addresses_use_base_128_at_kit_two_and_one_hundred() {
    for (kit, base) in [(2, [4, 2, 0, 0]), (100, [5, 0x46, 0, 0])] {
        let actual = Module::new(base).client().read_kit(kit).unwrap();
        assert_eq!(actual.blocks[0].address, base);
        assert_eq!(actual.kit, kit);
    }
    for kit in [0, 101] {
        let mut client = Module::new([4, 0, 0, 0]).client();
        assert!(client.read_kit(kit).is_err());
        assert!(client.into_inner().sent.is_empty());
    }
}

#[test]
fn split_dt1_messages_reassemble_the_complete_requested_range() {
    let mut io = Module::new([4, 0, 0, 0]);
    io.mode = ReplyMode::SplitMessages;
    assert_eq!(io.client().read_kit(1).unwrap(), baseline());
}

#[test]
fn out_of_order_dt1_messages_reassemble_by_their_addresses() {
    let mut io = Module::new([4, 0, 0, 0]);
    io.mode = ReplyMode::ReverseMessages;
    assert_eq!(io.client().read_kit(1).unwrap(), baseline());
}

#[test]
fn fragmented_receive_chunks_and_embedded_realtime_bytes_are_supported() {
    let mut io = Module::new([4, 0, 0, 0]);
    io.mode = ReplyMode::ByteFragments;
    assert_eq!(io.client().read_kit(1).unwrap(), baseline());
}

#[test]
fn invalid_response_checksum_device_model_address_and_seven_bit_data_are_rejected() {
    for mode in [
        ReplyMode::BadChecksum,
        ReplyMode::WrongDevice,
        ReplyMode::WrongModel,
        ReplyMode::WrongAddress,
        ReplyMode::NonSevenBit,
    ] {
        let mut io = Module::new([4, 0, 0, 0]);
        io.mode = mode;
        let mut client = io.client();
        assert!(client.read_kit(1).is_err());
        assert_eq!(client.into_inner().write_attempts, 0);
    }
}

#[test]
fn missing_bytes_and_overlapping_chunks_do_not_produce_a_snapshot() {
    for mode in [ReplyMode::MissingTail, ReplyMode::Overlap] {
        let mut io = Module::new([4, 0, 0, 0]);
        io.mode = mode;
        assert!(io.client().read_kit(1).is_err());
    }
}

#[test]
fn plan_rejects_unknown_keys_invalid_types_and_out_of_range_parameters() {
    let baseline = baseline();
    for (key, value) in [
        ("kit.unknown", json!(true)),
        ("pad.unknown.compressor_enabled", json!(true)),
        ("kit.name", json!("")),
        ("kit.name", json!("THIRTEENCHARS")),
        ("kit.name", json!("tést")),
        ("kit.name", json!("bad\nname")),
        ("kit.tempo", json!(19)),
        ("kit.tempo", json!(261)),
        ("kit.tempo", json!(120.5)),
        ("kit.tempo", json!("120")),
        ("kit.tempo_enabled", json!(1)),
        ("pad.snare.compressor_enabled", json!(null)),
    ] {
        assert!(
            td27::plan(&baseline, &change(key, value)).is_err(),
            "parameter {key}"
        );
    }
}

#[test]
fn planning_exposes_literal_addresses_nibbles_and_name_padding_without_writing() {
    let source = baseline();
    let changes = BTreeMap::from([
        ("kit.name".into(), json!("NEW TEST")),
        ("kit.tempo".into(), json!(260)),
        ("pad.snare.compressor_enabled".into(), json!(false)),
    ]);
    let plan = td27::plan(&source, &changes).unwrap();
    assert_eq!(
        serde_json::to_value(&plan.writes).unwrap(),
        json!([
            {"key":"kit.name","address":[4,0,0,0],"before":[84,69,83,84,32,75,73,84,32,65,32,32],"after":[78,69,87,32,84,69,83,84,32,32,32,32]},
            {"key":"kit.tempo","address":[4,0,0,46],"before":[0,0,7,11],"after":[0,1,0,4]},
            {"key":"pad.snare.compressor_enabled","address":[4,1,65,1],"before":[1],"after":[0]}
        ])
    );
    assert_eq!(source, baseline());
}

#[test]
fn apply_writes_only_planned_bytes_preserves_other_bytes_and_verifies_readback() {
    let source = baseline();
    let changes = BTreeMap::from([
        ("kit.name".into(), json!("NEW TEST")),
        ("kit.tempo".into(), json!(260)),
        ("pad.snare.compressor_enabled".into(), json!(false)),
    ]);
    let plan = td27::plan(&source, &changes).unwrap();
    let mut client = Module::new([4, 0, 0, 0]).client();
    let report = client.apply(&plan).unwrap();
    assert_eq!(report.status, "verified");
    assert_eq!((report.written, report.verified, report.total), (3, 3, 3));
    let actual = report.snapshot.unwrap();
    assert_eq!(actual.parameters["kit.name"], "NEW TEST");
    assert_eq!(actual.parameters["kit.tempo"], 260);
    assert_eq!(actual.parameters["pad.snare.compressor_enabled"], false);
    let io = client.into_inner();
    let mut expected = Module::new([4, 0, 0, 0]).memory;
    let base = number(&[4, 0, 0, 0]);
    for (offset, byte) in b"NEW TEST    ".iter().enumerate() {
        expected.insert(base + offset as u32, *byte);
    }
    for (offset, byte) in [0, 1, 0, 4].into_iter().enumerate() {
        expected.insert(base + 46 + offset as u32, byte);
    }
    expected.insert(number(&[4, 1, 65, 1]), 0);
    assert_eq!(io.memory, expected);
    assert_eq!(io.write_attempts, 3);
    assert!(
        io.waits
            .iter()
            .all(|wait| *wait >= Duration::from_millis(20))
    );
    assert_eq!(
        io.sent
            .iter()
            .filter(|packet| packet.get(7) == Some(&0x12))
            .count(),
        3
    );
}

#[test]
fn stale_scope_bytes_block_all_writes_even_when_an_unedited_field_changed() {
    let plan = td27::plan(&baseline(), &change("kit.tempo", json!(140))).unwrap();
    let mut io = Module::new([4, 0, 0, 0]);
    io.memory.insert(number(&[4, 0, 0, 0x10]), b'Z');
    let mut client = io.client();
    let report = client.apply(&plan).unwrap();
    assert_eq!(report.status, "conflict");
    assert_eq!((report.written, report.verified), (0, 0));
    assert_eq!(client.into_inner().write_attempts, 0);
}

#[test]
fn plan_for_another_device_is_rejected_before_a_write() {
    let plan = td27::plan(&baseline(), &change("kit.tempo", json!(140))).unwrap();
    let mut io = Module::new([4, 0, 0, 0]);
    io.device = 0x11;
    let mut client = io.client();
    let report = client.apply(&plan).unwrap();
    assert_eq!(report.status, "conflict");
    assert_eq!((report.written, report.verified), (0, 0));
    assert!(client.into_inner().sent.is_empty());
}

#[test]
fn forged_plan_writes_are_rejected_before_any_transport_activity() {
    let mut plan = td27::plan(&baseline(), &change("kit.tempo", json!(140))).unwrap();
    plan.writes[0].after = vec![0, 0, 9, 0];
    let mut client = Module::new([4, 0, 0, 0]).client();
    assert!(client.apply(&plan).is_err());
    assert!(client.into_inner().sent.is_empty());
}

#[test]
fn disconnect_after_one_write_stops_remaining_writes_and_reports_partial() {
    let changes = BTreeMap::from([
        ("kit.name".into(), json!("NEW TEST")),
        ("kit.tempo".into(), json!(140)),
        ("pad.snare.compressor_enabled".into(), json!(false)),
    ]);
    let plan = td27::plan(&baseline(), &changes).unwrap();
    let mut io = Module::new([4, 0, 0, 0]);
    io.fail_write = Some(2);
    let mut client = io.client();
    let report = client.apply(&plan).unwrap();
    assert_eq!(report.status, "partial");
    assert_eq!((report.written, report.verified, report.total), (1, 1, 3));
    assert_eq!(client.into_inner().write_attempts, 2);
}

#[test]
fn readback_mismatch_stops_remaining_writes_and_never_reports_verified() {
    let changes = BTreeMap::from([
        ("kit.tempo_enabled".into(), json!(false)),
        ("pad.snare.compressor_enabled".into(), json!(false)),
    ]);
    let plan = td27::plan(&baseline(), &changes).unwrap();
    let mut io = Module::new([4, 0, 0, 0]);
    io.mismatch = true;
    let mut client = io.client();
    let report = client.apply(&plan).unwrap();
    assert_eq!(report.status, "partial");
    assert_eq!((report.written, report.verified, report.total), (1, 0, 2));
    assert!(report.snapshot.is_none());
    assert_eq!(client.into_inner().write_attempts, 1);
}

#[test]
fn readback_transport_failure_preserves_uncertainty_and_stops_writes() {
    let plan = td27::plan(&baseline(), &change("kit.tempo_enabled", json!(false))).unwrap();
    let mut io = Module::new([4, 0, 0, 0]);
    io.fail_readback = true;
    let mut client = io.client();
    let report = client.apply(&plan).unwrap();
    assert_eq!(report.status, "partial");
    assert_eq!((report.written, report.verified), (1, 0));
    assert!(report.snapshot.is_none());
    assert_eq!(client.into_inner().write_attempts, 1);
}

#[test]
fn changed_native_endpoint_blocks_apply_before_any_packet_is_sent() {
    let mut source = baseline();
    source.endpoint = Some(MidiEndpoint {
        input: "Test input A".into(),
        output: "Test output A".into(),
    });
    let plan = td27::plan(&source, &change("kit.tempo", json!(140))).unwrap();
    let mut io = Module::new([4, 0, 0, 0]);
    io.endpoint = Some(MidiEndpoint {
        input: "Test input B".into(),
        output: "Test output B".into(),
    });
    let mut client = io.client();
    let report = client.apply(&plan).unwrap();
    assert_eq!(report.status, "conflict");
    assert_eq!((report.written, report.verified), (0, 0));
    assert!(client.into_inner().sent.is_empty());
}

#[test]
fn native_target_with_different_ports_fails_validation_before_opening_midi() {
    let mut source = baseline();
    source.endpoint = Some(MidiEndpoint {
        input: "KitControl nonexistent test input A".into(),
        output: "KitControl nonexistent test output A".into(),
    });
    let plan = td27::plan(&source, &change("kit.tempo", json!(140))).unwrap();
    let target = Target::Native {
        input: "KitControl nonexistent test input B".into(),
        output: "KitControl nonexistent test output B".into(),
        device_id: 0x10,
        firmware: "2.00".into(),
    };
    assert_eq!(
        td27_connection::apply(target, &plan).unwrap_err().code,
        "invalid_target"
    );
}

fn control_name_module() -> Module {
    let mut io = Module::new([4, 0, 0, 0]);
    for (offset, byte) in [1, 84, 69, 83, 84, 9, 65, 32, 32, 32, 32, 13]
        .into_iter()
        .enumerate()
    {
        io.memory
            .insert(number(&[4, 0, 0, 0]) + offset as u32, byte);
    }
    io
}

fn control_name_baseline() -> Td27Snapshot {
    let mut source = baseline();
    source.blocks[0].data[..12].copy_from_slice(&[1, 84, 69, 83, 84, 9, 65, 32, 32, 32, 32, 13]);
    source
        .parameters
        .insert("kit.name".into(), json!("\u{1}TEST\tA    \r"));
    source
}

#[test]
fn documented_ascii_name_bytes_are_read_without_trimming_control_characters() {
    let actual = control_name_module().client().read_kit(1).unwrap();
    assert_eq!(actual, control_name_baseline());
    assert_eq!(actual.parameters["kit.name"], "\u{1}TEST\tA    \r");
}

#[test]
fn unrelated_tempo_apply_preserves_documented_control_characters_in_the_name() {
    let plan = td27::plan(&control_name_baseline(), &change("kit.tempo", json!(140))).unwrap();
    let mut expected = control_name_module().memory;
    for (offset, byte) in [0, 0, 8, 12].into_iter().enumerate() {
        expected.insert(number(&[4, 0, 0, 0x2e]) + offset as u32, byte);
    }
    let mut client = control_name_module().client();
    let report = client.apply(&plan).unwrap();
    assert_eq!(report.status, "verified");
    assert_eq!((report.written, report.verified), (1, 1));
    assert_eq!(
        report.snapshot.unwrap().parameters["kit.name"],
        "\u{1}TEST\tA    \r"
    );
    assert_eq!(client.into_inner().memory, expected);
}

#[test]
fn final_read_failure_clears_verified_count_despite_successful_field_readback() {
    let plan = td27::plan(&baseline(), &change("kit.tempo_enabled", json!(false))).unwrap();
    let mut io = Module::new([4, 0, 0, 0]);
    io.fail_final_read = true;
    let report = io.client().apply(&plan).unwrap();
    assert_eq!(report.status, "partial");
    assert_eq!((report.written, report.verified, report.total), (1, 0, 1));
    assert!(report.snapshot.is_none());
}

#[test]
fn final_snapshot_mismatch_counts_actual_matching_fields_after_a_field_reverted() {
    let changes = BTreeMap::from([
        ("kit.tempo".into(), json!(140)),
        ("kit.tempo_enabled".into(), json!(false)),
    ]);
    let plan = td27::plan(&baseline(), &changes).unwrap();
    let mut io = Module::new([4, 0, 0, 0]);
    io.final_mismatch = true;
    let report = io.client().apply(&plan).unwrap();
    assert_eq!(report.status, "partial");
    assert_eq!((report.written, report.verified, report.total), (2, 1, 2));
    let actual = report.snapshot.unwrap();
    assert_eq!(actual.parameters["kit.tempo_enabled"], true);
    assert_eq!(actual.parameters["kit.tempo"], 140);
}

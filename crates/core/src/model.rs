use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

pub const SYNTHETIC_PROFILE: &str = "kitcontrol.synthetic.v1";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Document {
    pub format_version: u32,
    pub profile_id: String,
    pub document_id: String,
    pub kits: BTreeMap<String, Kit>,
    pub slots: Vec<String>,
    pub setlists: Vec<Setlist>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Kit {
    pub name: String,
    pub pads: BTreeMap<String, Pad>,
    pub effects: BTreeMap<String, Value>,
    pub opaque: BTreeMap<String, Vec<u8>>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pad {
    pub instrument: String,
    pub tuning: i32,
    pub muffling: i32,
    pub volume: i32,
    pub pan: i32,
    pub sub_instrument: Option<String>,
    pub layer_level: i32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Setlist {
    pub name: String,
    pub slots: Vec<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeviceIdentity {
    pub device_id: String,
    pub profile_id: String,
    pub simulated: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub identity: DeviceIdentity,
    pub document: Document,
}

#[derive(Clone, Debug, Serialize)]
pub struct DeviceProfile {
    pub id: String,
    pub name: String,
    pub simulated: bool,
    pub hardware_writes: bool,
    pub instruments: Vec<String>,
}

pub fn synthetic_profile() -> DeviceProfile {
    DeviceProfile {
        id: SYNTHETIC_PROFILE.into(),
        name: "KitControl synthetic device".into(),
        simulated: true,
        hardware_writes: false,
        instruments: [
            "synth.kick.01",
            "synth.snare.01",
            "synth.tom.01",
            "synth.cymbal.01",
        ]
        .map(String::from)
        .to_vec(),
    }
}

pub fn validate_id(value: &str) -> Result<()> {
    if value.is_empty()
        || value.len() > 64
        || !value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        return Err(Error::new(
            "invalid_id",
            "IDs must contain 1–64 ASCII letters, digits, hyphens or underscores",
        ));
    }
    Ok(())
}

pub fn validate_name(value: &str) -> Result<()> {
    if value.trim().is_empty() || value.chars().count() > 64 || value.chars().any(char::is_control)
    {
        return Err(Error::new(
            "invalid_name",
            "Names must contain 1–64 visible characters",
        ));
    }
    Ok(())
}

pub fn validate_instrument(value: &str) -> Result<()> {
    if !synthetic_profile().instruments.iter().any(|i| i == value) {
        return Err(Error::new(
            "unsupported_instrument",
            format!("Unknown synthetic instrument: {value}"),
        ));
    }
    Ok(())
}

impl Pad {
    pub fn validate(&self) -> Result<()> {
        validate_instrument(&self.instrument)?;
        if let Some(sub) = &self.sub_instrument {
            validate_instrument(sub)?;
        }
        for (name, value, low, high) in [
            ("tuning", self.tuning, -100, 100),
            ("muffling", self.muffling, 0, 10),
            ("volume", self.volume, 0, 100),
            ("pan", self.pan, -30, 30),
            ("layer_level", self.layer_level, 0, 100),
        ] {
            if !(low..=high).contains(&value) {
                return Err(Error::new(
                    "out_of_range",
                    format!("{name} must be between {low} and {high}"),
                ));
            }
        }
        Ok(())
    }
}

impl Document {
    pub fn validate(&self) -> Result<()> {
        if self.format_version != 1 {
            return Err(Error::new(
                "unsupported_version",
                "Only document format version 1 is supported",
            ));
        }
        if self.profile_id != SYNTHETIC_PROFILE {
            return Err(Error::new(
                "hardware_locked",
                "Only the synthetic profile is available. Roland support awaits legal and hardware review.",
            ));
        }
        validate_id(&self.document_id)?;
        if self.kits.is_empty()
            || self.kits.len() > 1024
            || self.slots.is_empty()
            || self.slots.len() > 128
        {
            return Err(Error::new(
                "invalid_document",
                "A document needs 1–1024 kits and 1–128 occupied slots",
            ));
        }
        for (id, kit) in &self.kits {
            validate_id(id)?;
            validate_name(&kit.name)?;
            if kit.pads.is_empty() || kit.pads.len() > 64 {
                return Err(Error::new("invalid_document", "A kit needs 1–64 pad zones"));
            }
            for (zone, pad) in &kit.pads {
                if zone.is_empty()
                    || zone.len() > 64
                    || !zone
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
                {
                    return Err(Error::new("invalid_zone", "Invalid pad zone identifier"));
                }
                pad.validate()?;
            }
        }
        let mut seen = BTreeSet::new();
        for id in &self.slots {
            if !self.kits.contains_key(id) || !seen.insert(id) {
                return Err(Error::new(
                    "invalid_slots",
                    "Each slot must reference a distinct existing kit",
                ));
            }
        }
        for list in &self.setlists {
            validate_name(&list.name)?;
            if list
                .slots
                .iter()
                .any(|slot| *slot == 0 || *slot > self.slots.len())
            {
                return Err(Error::new(
                    "invalid_setlist",
                    "Setlist slots are one-based and must exist",
                ));
            }
        }
        Ok(())
    }
}

impl Snapshot {
    pub fn validate(&self) -> Result<()> {
        self.document.validate()?;
        validate_id(&self.identity.device_id)?;
        if !self.identity.simulated || self.identity.profile_id != self.document.profile_id {
            return Err(Error::new(
                "hardware_locked",
                "A matching synthetic device identity is required",
            ));
        }
        Ok(())
    }
}

pub fn demo_document() -> Document {
    let kits = ["Studio", "Dry", "Room", "Practice"]
        .into_iter()
        .enumerate()
        .map(|(index, name)| {
            let pads = [
                ("kick.head", "synth.kick.01"),
                ("snare.head", "synth.snare.01"),
                ("snare.rim", "synth.snare.01"),
                ("tom1.head", "synth.tom.01"),
            ]
            .into_iter()
            .map(|(zone, instrument)| {
                (
                    zone.into(),
                    Pad {
                        instrument: instrument.into(),
                        tuning: 0,
                        muffling: index as i32,
                        volume: 80,
                        pan: 0,
                        sub_instrument: None,
                        layer_level: 0,
                    },
                )
            })
            .collect();
            (
                format!("kit-{}", index + 1),
                Kit {
                    name: name.into(),
                    pads,
                    effects: BTreeMap::from([("room".into(), Value::from(index * 10))]),
                    opaque: BTreeMap::new(),
                },
            )
        })
        .collect();
    Document {
        format_version: 1,
        profile_id: SYNTHETIC_PROFILE.into(),
        document_id: "synthetic-workspace".into(),
        kits,
        slots: (1..=4).map(|i| format!("kit-{i}")).collect(),
        setlists: vec![Setlist {
            name: "Practice set".into(),
            slots: vec![1, 3, 2],
        }],
    }
}

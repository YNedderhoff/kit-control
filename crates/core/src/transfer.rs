use crate::{
    error::{Error, Result},
    model::{DeviceIdentity, Document, Snapshot},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Change {
    pub path: String,
    pub before: Option<Value>,
    pub after: Option<Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChangePlan {
    pub format_version: u32,
    pub expected_device: DeviceIdentity,
    pub baseline_hash: String,
    pub desired_hash: String,
    pub baseline: Document,
    pub desired: Document,
    pub changes: Vec<Change>,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Fault {
    #[default]
    None,
    Disconnect,
    ReadbackMismatch,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApplyReport {
    pub status: String,
    pub written: usize,
    pub verified: usize,
    pub total: usize,
    pub message: String,
    pub snapshot: Snapshot,
    pub changes: Vec<Change>,
}

pub fn fingerprint(document: &Document) -> Result<String> {
    document.validate()?;
    Ok(format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(document)?)
    ))
}

pub fn diff(before: &Document, after: &Document) -> Result<Vec<Change>> {
    before.validate()?;
    after.validate()?;
    let mut result = Vec::new();
    compare(
        "",
        Some(&serde_json::to_value(before)?),
        Some(&serde_json::to_value(after)?),
        &mut result,
    );
    Ok(result)
}

fn compare(path: &str, before: Option<&Value>, after: Option<&Value>, changes: &mut Vec<Change>) {
    if before == after {
        return;
    }
    if let (Some(Value::Object(left)), Some(Value::Object(right))) = (before, after) {
        for key in left.keys().chain(right.keys()).collect::<BTreeSet<_>>() {
            let segment = key.replace('~', "~0").replace('/', "~1");
            compare(
                &format!("{path}/{segment}"),
                left.get(key),
                right.get(key),
                changes,
            );
        }
    } else {
        changes.push(Change {
            path: path.into(),
            before: before.cloned(),
            after: after.cloned(),
        });
    }
}

pub fn plan(baseline: &Snapshot, desired: &Document) -> Result<ChangePlan> {
    baseline.validate()?;
    desired.validate()?;
    if desired.document_id != baseline.document.document_id
        || desired.profile_id != baseline.document.profile_id
    {
        return Err(Error::new(
            "incompatible_document",
            "Baseline and desired document must have the same identity and profile",
        ));
    }
    Ok(ChangePlan {
        format_version: 1,
        expected_device: baseline.identity.clone(),
        baseline_hash: fingerprint(&baseline.document)?,
        desired_hash: fingerprint(desired)?,
        baseline: baseline.document.clone(),
        desired: desired.clone(),
        changes: diff(&baseline.document, desired)?,
    })
}

impl ChangePlan {
    pub fn validate(&self) -> Result<()> {
        let baseline = Snapshot {
            identity: self.expected_device.clone(),
            document: self.baseline.clone(),
        };
        let expected = plan(&baseline, &self.desired)?;
        if self.format_version != 1
            || self.baseline_hash != expected.baseline_hash
            || self.desired_hash != expected.desired_hash
            || self.changes != expected.changes
        {
            return Err(Error::new(
                "invalid_plan",
                "Plan metadata or changes do not match its documents",
            ));
        }
        Ok(())
    }
}

// This transport models document operations, not Roland MIDI packets. Protocol
// simulation is gated separately and must not be inferred from these tests.
pub trait DeviceTransport {
    fn read(&self) -> Result<Snapshot>;
    fn write(&mut self, desired: &Document, fault: Fault) -> Result<()>;
}

pub struct Simulator {
    snapshot: Snapshot,
}

impl Simulator {
    pub fn new(snapshot: Snapshot) -> Result<Self> {
        snapshot.validate()?;
        Ok(Self { snapshot })
    }
}

impl DeviceTransport for Simulator {
    fn read(&self) -> Result<Snapshot> {
        Ok(self.snapshot.clone())
    }
    fn write(&mut self, desired: &Document, fault: Fault) -> Result<()> {
        if fault == Fault::Disconnect {
            return Err(Error::new(
                "disconnected",
                "Injected simulator disconnect before document write",
            ));
        }
        self.snapshot.document = desired.clone();
        if fault == Fault::ReadbackMismatch {
            let kit = self
                .snapshot
                .document
                .kits
                .values_mut()
                .next()
                .expect("validated nonempty");
            kit.name = if kit.name == "Injected mismatch" {
                "Injected mismatch 2"
            } else {
                "Injected mismatch"
            }
            .into();
        }
        Ok(())
    }
}

pub fn apply(
    transport: &mut impl DeviceTransport,
    change_plan: &ChangePlan,
    fault: Fault,
) -> Result<ApplyReport> {
    change_plan.validate()?;
    let initial = transport.read()?;
    initial.validate()?;
    if initial.identity != change_plan.expected_device
        || fingerprint(&initial.document)? != change_plan.baseline_hash
    {
        return Ok(ApplyReport {
            status: "conflict".into(),
            written: 0,
            verified: 0,
            total: change_plan.changes.len(),
            message: "Device identity or baseline changed; create a new plan".into(),
            snapshot: initial,
            changes: Vec::new(),
        });
    }
    if change_plan.changes.is_empty() {
        return Ok(ApplyReport {
            status: "verified".into(),
            written: 0,
            verified: 0,
            total: 0,
            message: "No changes to apply".into(),
            snapshot: initial,
            changes: Vec::new(),
        });
    }
    if let Err(error) = transport.write(&change_plan.desired, fault) {
        return Ok(ApplyReport {
            status: "failed".into(),
            written: 0,
            verified: 0,
            total: change_plan.changes.len(),
            message: error.message,
            snapshot: transport.read()?,
            changes: Vec::new(),
        });
    }
    let actual = transport.read()?;
    let differences = diff(&actual.document, &change_plan.desired)?;
    let matches = actual.identity == change_plan.expected_device && differences.is_empty();
    Ok(ApplyReport {
        status: if matches { "verified" } else { "partial" }.into(),
        written: change_plan.changes.len(),
        verified: if matches {
            change_plan.changes.len()
        } else {
            0
        },
        total: change_plan.changes.len(),
        message: if matches {
            "Simulator document written and read back"
        } else {
            "Readback differs; device state is not verified. Read again before recovery."
        }
        .into(),
        snapshot: actual,
        changes: differences,
    })
}

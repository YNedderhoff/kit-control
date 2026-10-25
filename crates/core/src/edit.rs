use crate::{
    error::{Error, Result},
    model::{Document, validate_id, validate_name},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum EditOperation {
    RenameKit {
        kit_id: String,
        name: String,
    },
    SetParameter {
        kit_id: String,
        pad: String,
        parameter: String,
        value: Value,
    },
    CopyPad {
        source_kit_id: String,
        source_pad: String,
        target_kit_id: String,
        target_pad: String,
    },
    CopyKit {
        source_kit_id: String,
        target_slot: usize,
        new_kit_id: String,
    },
    ReorderKits {
        order: Vec<String>,
    },
}

pub fn edit(document: &Document, operation: EditOperation) -> Result<Document> {
    document.validate()?;
    let mut next = document.clone();
    match operation {
        EditOperation::RenameKit { kit_id, name } => {
            validate_name(&name)?;
            next.kits
                .get_mut(&kit_id)
                .ok_or_else(|| Error::new("unknown_kit", kit_id))?
                .name = name;
        }
        EditOperation::SetParameter {
            kit_id,
            pad,
            parameter,
            value,
        } => {
            let target = next
                .kits
                .get_mut(&kit_id)
                .ok_or_else(|| Error::new("unknown_kit", kit_id))?
                .pads
                .get_mut(&pad)
                .ok_or_else(|| Error::new("unknown_pad", pad))?;
            match parameter.as_str() {
                "instrument" => {
                    target.instrument = value
                        .as_str()
                        .ok_or_else(|| Error::new("invalid_value", "instrument must be a string"))?
                        .into()
                }
                "sub_instrument" => {
                    target.sub_instrument = if value.is_null() {
                        None
                    } else {
                        Some(
                            value
                                .as_str()
                                .ok_or_else(|| {
                                    Error::new(
                                        "invalid_value",
                                        "sub_instrument must be a string or null",
                                    )
                                })?
                                .into(),
                        )
                    }
                }
                "tuning" | "muffling" | "volume" | "pan" | "layer_level" => {
                    let number = value
                        .as_i64()
                        .and_then(|n| i32::try_from(n).ok())
                        .ok_or_else(|| {
                            Error::new(
                                "invalid_value",
                                "Numeric parameters require a 32-bit integer",
                            )
                        })?;
                    match parameter.as_str() {
                        "tuning" => target.tuning = number,
                        "muffling" => target.muffling = number,
                        "volume" => target.volume = number,
                        "pan" => target.pan = number,
                        _ => target.layer_level = number,
                    }
                }
                _ => return Err(Error::new("unknown_parameter", parameter)),
            }
        }
        EditOperation::CopyPad {
            source_kit_id,
            source_pad,
            target_kit_id,
            target_pad,
        } => {
            let source = next
                .kits
                .get(&source_kit_id)
                .ok_or_else(|| Error::new("unknown_kit", source_kit_id))?
                .pads
                .get(&source_pad)
                .ok_or_else(|| Error::new("unknown_pad", source_pad))?
                .clone();
            let target = next
                .kits
                .get_mut(&target_kit_id)
                .ok_or_else(|| Error::new("unknown_kit", target_kit_id))?
                .pads
                .get_mut(&target_pad)
                .ok_or_else(|| Error::new("unknown_pad", target_pad))?;
            *target = source;
        }
        EditOperation::CopyKit {
            source_kit_id,
            target_slot,
            new_kit_id,
        } => {
            validate_id(&new_kit_id)?;
            if next.kits.contains_key(&new_kit_id) {
                return Err(Error::new("duplicate_kit", "The new kit ID already exists"));
            }
            let source = next
                .kits
                .get(&source_kit_id)
                .ok_or_else(|| Error::new("unknown_kit", source_kit_id))?
                .clone();
            let target = next
                .slots
                .get_mut(
                    target_slot
                        .checked_sub(1)
                        .ok_or_else(|| Error::new("invalid_slot", "Slots are one-based"))?,
                )
                .ok_or_else(|| Error::new("invalid_slot", "Target slot does not exist"))?;
            *target = new_kit_id.clone();
            next.kits.insert(new_kit_id, source);
        }
        EditOperation::ReorderKits { order } => {
            let previous: BTreeSet<_> = next.slots.iter().collect();
            let requested: BTreeSet<_> = order.iter().collect();
            if order.len() != next.slots.len() || previous != requested {
                return Err(Error::new(
                    "invalid_order",
                    "Order must be an exact permutation of the occupied kit IDs",
                ));
            }
            for list in &mut next.setlists {
                for slot in &mut list.slots {
                    let id = &next.slots[*slot - 1];
                    *slot = order
                        .iter()
                        .position(|candidate| candidate == id)
                        .expect("permutation checked")
                        + 1;
                }
            }
            next.slots = order;
        }
    }
    next.validate()?;
    Ok(next)
}

use super::{ChangesReport, KeyChanges};
use serde::{Deserialize, Deserializer};
use std::collections::BTreeMap;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct WireReport {
    schema_version: u32,
    #[serde(flatten)]
    categories: BTreeMap<String, ChangeLists>,
    unclassified: Vec<String>,
    complete: bool,
    #[serde(default)]
    reasons: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ChangeLists {
    #[serde(default, deserialize_with = "present_list")]
    changed: Option<Vec<String>>,
    #[serde(default, deserialize_with = "present_list")]
    added: Option<Vec<String>>,
    #[serde(default, deserialize_with = "present_list")]
    removed: Option<Vec<String>>,
}

fn present_list<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Vec<String>>, D::Error> {
    Vec::deserialize(deserializer).map(Some)
}

impl TryFrom<WireReport> for ChangesReport {
    type Error = String;

    fn try_from(wire: WireReport) -> Result<Self, Self::Error> {
        let mut categories = BTreeMap::new();
        for (name, lists) in wire.categories {
            if lists.changed.is_none() && lists.added.is_none() && lists.removed.is_none() {
                return Err(format!("changes category {name:?} requires a change list"));
            }
            categories.insert(
                name,
                KeyChanges {
                    changed: lists.changed.unwrap_or_default(),
                    added: lists.added.unwrap_or_default(),
                    removed: lists.removed.unwrap_or_default(),
                },
            );
        }
        Ok(Self {
            schema_version: wire.schema_version,
            categories,
            unclassified: wire.unclassified,
            complete: wire.complete,
            reasons: wire.reasons,
        })
    }
}

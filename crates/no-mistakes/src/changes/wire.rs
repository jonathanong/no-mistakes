use super::{attributes, CategoryChanges, ChangesReport};
use serde::{Deserialize, Deserializer};
use std::collections::{BTreeMap, BTreeSet};

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
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ChangeLists {
    #[serde(default, deserialize_with = "present_list")]
    changed: Option<Vec<String>>,
    #[serde(default, deserialize_with = "present_list")]
    added: Option<Vec<String>>,
    #[serde(default, deserialize_with = "present_list")]
    removed: Option<Vec<String>>,
    #[serde(default)]
    attribute_paths: BTreeMap<String, Vec<String>>,
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
            let changed = lists.changed.unwrap_or_default();
            let added = lists.added.unwrap_or_default();
            let removed = lists.removed.unwrap_or_default();
            let mut attribute_paths = lists.attribute_paths;
            if !attribute_paths.is_empty() {
                let changed_only = changed.iter().collect::<BTreeSet<_>>();
                let other = added.iter().chain(&removed).collect::<BTreeSet<_>>();
                attribute_paths.retain(|id, paths| {
                    changed_only.contains(id)
                        && !other.contains(id)
                        && !paths.is_empty()
                        && paths.iter().all(|path| attributes::valid_pointer(path))
                });
            }
            categories.insert(
                name,
                CategoryChanges {
                    changed,
                    added,
                    removed,
                    attribute_paths,
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

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod tests;

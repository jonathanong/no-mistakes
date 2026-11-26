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
    #[serde(default)]
    changed_files: Vec<String>,
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
    #[serde(default, deserialize_with = "unique_attribute_paths")]
    attribute_paths: BTreeMap<String, Vec<String>>,
}

fn present_list<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Vec<String>>, D::Error> {
    Vec::deserialize(deserializer).map(Some)
}

fn unique_attribute_paths<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<BTreeMap<String, Vec<String>>, D::Error> {
    struct UniqueMap;

    impl<'de> serde::de::Visitor<'de> for UniqueMap {
        type Value = BTreeMap<String, Vec<String>>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("a map of unique fixture IDs to JSON pointer lists")
        }

        fn visit_map<A: serde::de::MapAccess<'de>>(
            self,
            mut map: A,
        ) -> Result<Self::Value, A::Error> {
            let mut entries = BTreeMap::new();
            while let Some((id, paths)) = map.next_entry::<String, Vec<String>>()? {
                if entries.insert(id.clone(), paths).is_some() {
                    return Err(serde::de::Error::custom(format!(
                        "duplicate attributePaths key {id:?}"
                    )));
                }
            }
            Ok(entries)
        }
    }

    deserializer.deserialize_map(UniqueMap)
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
            changed_files: wire.changed_files,
            complete: wire.complete,
            reasons: wire.reasons,
        })
    }
}

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod tests;

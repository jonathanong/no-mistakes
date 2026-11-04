use super::super::schema::ConfigProvenance;
use crate::config::source_locations::ConfigPath;
use serde_yaml::Value;
use std::path::Path;

pub(super) fn record_provenance(
    value: &Value,
    global: ConfigPath,
    local: ConfigPath,
    existing: Option<&Value>,
    path: &Path,
    provenance: &mut ConfigProvenance,
) {
    provenance.insert(global.clone(), path.to_path_buf(), local.clone());
    match value {
        Value::Mapping(mapping) => {
            for (key, value) in mapping {
                if let Some(key) = key.as_str() {
                    let canonical = super::aliases::canonical_key(&global, key);
                    if canonical != key {
                        provenance.insert(
                            global.clone().key(key),
                            path.to_path_buf(),
                            local.clone().key(key),
                        );
                    }
                    record_provenance(
                        value,
                        global.clone().key(&canonical),
                        local.clone().key(key),
                        existing
                            .and_then(Value::as_mapping)
                            .and_then(|map| map.get(Value::String(canonical))),
                        path,
                        provenance,
                    );
                }
            }
        }
        Value::Sequence(items) => {
            let global = if matches!(global.as_slice().last(), Some(crate::config::source_locations::Segment::Key(key)) if key == "fullSuiteTriggers")
            {
                global.key("triggers")
            } else {
                global
            };
            let offset = existing.and_then(Value::as_sequence).map_or(0, Vec::len);
            for (index, value) in items.iter().enumerate() {
                record_provenance(
                    value,
                    global.clone().index(index + offset),
                    local.clone().index(index),
                    None,
                    path,
                    provenance,
                );
            }
        }
        _ => {}
    }
}

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
                    record_provenance(
                        value,
                        global.clone().key(key),
                        local.clone().key(key),
                        existing
                            .and_then(Value::as_mapping)
                            .and_then(|map| map.get(Value::String(key.to_string()))),
                        path,
                        provenance,
                    );
                }
            }
        }
        Value::Sequence(items) => {
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

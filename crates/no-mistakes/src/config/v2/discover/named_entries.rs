use anyhow::Result;
use serde_yaml::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Default)]
pub(super) struct NamedEntries(BTreeMap<(String, String), PathBuf>);

impl NamedEntries {
    pub(super) fn check(
        &mut self,
        value: &Value,
        key: &str,
        path: &Path,
        existing: Option<&Value>,
    ) -> Result<()> {
        match value {
            Value::Mapping(map) => {
                for (name, value) in map {
                    if let Some(name) = name.as_str() {
                        self.check(
                            value,
                            &format!("{key}/{name}"),
                            path,
                            existing
                                .and_then(Value::as_mapping)
                                .and_then(|map| map.get(name)),
                        )?;
                    }
                }
            }
            Value::Sequence(items) => {
                let offset = existing.and_then(Value::as_sequence).map_or(0, Vec::len);
                for (index, value) in items.iter().enumerate() {
                    if let Some(name) = value
                        .as_mapping()
                        .and_then(|map| map.get("name"))
                        .and_then(Value::as_str)
                    {
                        let entry = (key.to_string(), name.to_string());
                        if let Some(previous) = self.0.get(&entry) {
                            if previous != path {
                                anyhow::bail!(
                                    "duplicate name `{name}` in {key}: {} and {}",
                                    previous.display(),
                                    path.display()
                                );
                            }
                        } else {
                            self.0.insert(entry, path.to_path_buf());
                        }
                    }
                    self.check(value, &format!("{key}/{}", index + offset), path, None)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
}

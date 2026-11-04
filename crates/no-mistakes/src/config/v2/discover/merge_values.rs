use anyhow::Result;
use serde_yaml::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub(super) fn merge(
    target: &mut Value,
    incoming: Value,
    key: &str,
    path: &Path,
    origins: &mut BTreeMap<String, PathBuf>,
) -> Result<()> {
    match (target, incoming) {
        (Value::Mapping(target), Value::Mapping(incoming)) => {
            for (name, value) in incoming {
                let child = format!("{key}/{}", name.as_str().unwrap_or("?"));
                if let Some(existing) = target.get_mut(&name) {
                    merge(existing, value, &child, path, origins)?;
                } else {
                    record(&value, &child, path, origins);
                    target.insert(name, value);
                }
            }
            Ok(())
        }
        (Value::Sequence(target), Value::Sequence(mut incoming)) => {
            target.append(&mut incoming);
            Ok(())
        }
        (_, _) => {
            let previous = origins
                .get(key)
                .map_or_else(|| "unknown".to_string(), |p| p.display().to_string());
            anyhow::bail!(
                "conflicting config value `{key}` in {previous} and {}",
                path.display()
            )
        }
    }
}

fn record(value: &Value, key: &str, path: &Path, origins: &mut BTreeMap<String, PathBuf>) {
    origins.insert(key.to_string(), path.to_path_buf());
    if let Value::Mapping(mapping) = value {
        for (name, value) in mapping {
            record(
                value,
                &format!("{key}/{}", name.as_str().unwrap_or("?")),
                path,
                origins,
            );
        }
    }
}

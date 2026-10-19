use anyhow::{bail, Result};
use serde_yaml::{Mapping, Value};

use super::super::schema::ProjectPostgresTrustedSqlTag;

pub(super) fn ensure_mapping<'a>(options: &'a mut Value, rule: &str) -> Result<&'a mut Mapping> {
    if options.is_null() {
        *options = Value::Mapping(Mapping::new());
    }
    options
        .as_mapping_mut()
        .ok_or_else(|| anyhow::anyhow!("{rule} options must be a mapping"))
}

pub(super) fn string_list(value: Option<&Value>, rule: &str, key: &str) -> Result<Vec<String>> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    if value.is_null() {
        return Ok(Vec::new());
    }
    let Some(items) = value.as_sequence() else {
        bail!("{rule} options.{key} must be a list of globs");
    };
    items
        .iter()
        .map(|item| {
            item.as_str()
                .map(str::to_string)
                .ok_or_else(|| anyhow::anyhow!("{rule} options.{key} must be a list of globs"))
        })
        .collect()
}

/// Catalog rules deny unknown fields and only read `schemaCatalogPath`.
pub(super) fn write_catalog_only(rule: &str, options: &mut Mapping, path: &str) -> bool {
    if !matches!(rule, "postgres-column-naming" | "postgres-key-column-types") {
        return false;
    }
    if !path.is_empty() {
        set_string(options, "schemaCatalogPath", path);
    }
    true
}

pub(super) fn set_string(map: &mut Mapping, key: &str, value: &str) {
    map.insert(
        Value::String(key.to_string()),
        Value::String(value.to_string()),
    );
}

pub(super) fn set_bool(map: &mut Mapping, key: &str, value: bool) {
    map.insert(Value::String(key.to_string()), Value::Bool(value));
}

pub(super) fn set_strings(map: &mut Mapping, key: &str, values: &[String]) {
    let items = values
        .iter()
        .cloned()
        .map(Value::String)
        .collect::<Vec<_>>();
    map.insert(Value::String(key.to_string()), Value::Sequence(items));
}

pub(super) fn set_tags(map: &mut Mapping, tags: &[ProjectPostgresTrustedSqlTag]) {
    let items = tags
        .iter()
        .map(|tag| {
            let mut mapping = Mapping::new();
            mapping.insert(
                Value::String("module".to_string()),
                Value::String(tag.module.clone()),
            );
            mapping.insert(
                Value::String("name".to_string()),
                Value::String(tag.name.clone()),
            );
            Value::Mapping(mapping)
        })
        .collect();
    map.insert(
        Value::String("trustedSqlTags".to_string()),
        Value::Sequence(items),
    );
}

pub(super) fn contains_key(map: &Mapping, key: &str) -> bool {
    map.contains_key(Value::String(key.to_string()))
}

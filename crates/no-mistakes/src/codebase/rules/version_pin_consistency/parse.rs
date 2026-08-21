use crate::codebase::ts_source::relative_slash_path;
use serde_yaml::Value;
use std::path::{Path, PathBuf};

pub(super) fn configured_rel(rel: &str) -> &str {
    rel.trim_start_matches("./")
}

pub(super) fn tracked_rels(root: &Path, files: &[PathBuf]) -> std::collections::HashSet<String> {
    files
        .iter()
        .map(|path| configured_rel(&relative_slash_path(root, path)).to_string())
        .collect()
}

pub(super) fn parse_source(path: &Path, source: &str) -> Result<Value, String> {
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or("");
    if extension.eq_ignore_ascii_case("toml") {
        parse_toml(source)
    } else {
        crate::codebase::structured_value::parse_structured_value(path, source)
    }
}

fn parse_toml(source: &str) -> Result<Value, String> {
    if source.trim().is_empty() {
        return Ok(Value::Mapping(serde_yaml::Mapping::new()));
    }
    let parsed: toml::Value =
        toml::from_str(source).map_err(|error| format!("failed to parse TOML ({error})"))?;
    serde_yaml::to_value(parsed).map_err(|error| format!("failed to parse TOML ({error})"))
}

/// Resolve `section.key` where `key` may contain `:`, `/`, and extra `.`.
pub(super) fn value_at_key<'a>(value: &'a Value, key: &str) -> Option<&'a Value> {
    let Some((section, rest)) = key.split_once('.') else {
        return value.get(key);
    };
    let child = value.get(section)?;
    child.get(rest).or_else(|| {
        rest.split('.')
            .try_fold(child, |current, part| current.get(part))
    })
}

pub(super) fn pin_kind(value: &Value) -> String {
    match value {
        Value::Number(number) => number.to_string(),
        Value::Bool(flag) => flag.to_string(),
        Value::Null => "null".to_string(),
        Value::Sequence(_) => "array".to_string(),
        Value::Mapping(_) => "object".to_string(),
        Value::Tagged(tagged) => pin_kind(&tagged.value),
        Value::String(text) => format!("\"{text}\""),
    }
}

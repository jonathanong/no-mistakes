use crate::codebase::ts_source::relative_slash_path;
use serde_yaml::Value;
use std::path::{Path, PathBuf};

pub(super) fn read_text(
    root: &Path,
    rel: &str,
    sources: &crate::codebase::ts_source::SourceStore,
) -> String {
    let path = root.join(rel);
    super::super::read_source(sources, &path)
        .map(|source| source.to_string())
        .unwrap_or_else(|| std::fs::read_to_string(path).unwrap_or_default())
}

pub(super) fn tracked_rels(root: &Path, files: &[PathBuf]) -> std::collections::HashSet<String> {
    files
        .iter()
        .map(|path| relative_slash_path(root, path))
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

pub(super) fn key_line(source: &str, source_key: &str) -> usize {
    let needle = source_key
        .rsplit_once('.')
        .map_or(source_key, |(_, key)| key);
    source
        .lines()
        .position(|line| line.contains(needle))
        .map_or(1, |index| index + 1)
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

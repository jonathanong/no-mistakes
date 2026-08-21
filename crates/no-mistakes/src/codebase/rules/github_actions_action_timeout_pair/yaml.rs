use super::{RuleFinding, UsesSpec, RULE_ID};
use crate::codebase::ts_source::relative_slash_path;
use serde_yaml::{Mapping, Value};
use std::path::{Path, PathBuf};

pub(super) fn mapping_get<'a>(map: &'a Mapping, key: &str) -> Option<&'a Value> {
    map.get(Value::String(key.to_string()))
}

pub(super) fn mapping_string(step: &Mapping, key: &str) -> Option<String> {
    mapping_get(step, key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

pub(super) fn step_label(step: &Mapping, index: usize) -> String {
    mapping_string(step, "name")
        .or_else(|| mapping_string(step, "id"))
        .or_else(|| mapping_string(step, "uses"))
        .unwrap_or_else(|| format!("step #{}", index + 1))
}

pub(super) fn finding(file: &str, line: usize, message: String, target: &str) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line,
        message,
        import: None,
        target: Some(target.to_string()),
    }
}

pub(super) fn literal_u64(value: &Value) -> Option<u64> {
    match value {
        Value::Number(number) => number
            .as_u64()
            .or_else(|| number.as_i64().and_then(|value| u64::try_from(value).ok())),
        _ => None,
    }
}

pub(super) fn yaml_got(value: Option<&Value>) -> String {
    match value {
        None | Some(Value::Null) => "null".to_string(),
        Some(Value::Number(number)) => number.to_string(),
        Some(Value::String(text)) => format!("\"{text}\""),
        Some(Value::Bool(flag)) => flag.to_string(),
        Some(_) => "non-literal".to_string(),
    }
}

pub(super) fn normalize_uses(uses: &str) -> String {
    let trimmed = uses.trim();
    trimmed.strip_suffix('/').unwrap_or(trimmed).to_string()
}

pub(super) fn uses_matches(uses: &str, specs: &[UsesSpec]) -> bool {
    let normalized = normalize_uses(uses);
    specs.iter().any(|spec| match spec {
        UsesSpec::Exact(exact) => normalized == *exact,
        UsesSpec::Prefix(prefix) => normalized.to_ascii_lowercase().starts_with(prefix),
    })
}

pub(super) fn is_direct_third_party(uses: &str, specs: &[UsesSpec]) -> bool {
    let normalized = normalize_uses(uses);
    if normalized.starts_with("./") {
        return false;
    }
    specs.iter().any(|spec| match spec {
        UsesSpec::Prefix(prefix) => normalized.to_ascii_lowercase().starts_with(prefix),
        UsesSpec::Exact(_) => false,
    })
}

pub(super) fn is_local_wrapper(rel: &str, specs: &[UsesSpec]) -> bool {
    specs.iter().any(|spec| match spec {
        UsesSpec::Exact(path) => path
            .strip_prefix("./")
            .is_some_and(|local| is_wrapper_action_file(rel, local)),
        UsesSpec::Prefix(_) => false,
    })
}

pub(super) fn wrapper_rel(root: &Path, path: &Path, target_roots: &[PathBuf]) -> String {
    target_roots
        .iter()
        .map(PathBuf::as_path)
        .chain(std::iter::once(root))
        .filter(|base| path.starts_with(base))
        .max_by_key(|base| base.as_os_str().len())
        .map(|base| relative_slash_path(base, path))
        .unwrap_or_else(|| relative_slash_path(root, path))
}

fn is_wrapper_action_file(rel: &str, local: &str) -> bool {
    let rest = if rel == local {
        rel.rsplit('/').next().unwrap_or(rel)
    } else if let Some(rest) = rel
        .strip_prefix(local)
        .and_then(|rest| rest.strip_prefix('/'))
    {
        rest
    } else {
        return false;
    };
    matches!(rest, "action.yml" | "action.yaml")
}

pub(super) fn timeout_message(
    rel: &str,
    job_id: &str,
    label: &str,
    expected: u64,
    got: &str,
) -> String {
    format!(
        "{rel}: job \"{job_id}\" step \"{label}\" uses a configured action and must set timeout-minutes: {expected}; got {got}"
    )
}

pub(super) fn nested_message(
    rel: &str,
    job_id: &str,
    label: &str,
    input: &str,
    expected: u64,
    got: &str,
) -> String {
    format!(
        "{rel}: job \"{job_id}\" step \"{label}\" calls a third-party action directly and must pass {input}: {expected} (a bare number, not a quoted string) in its with: block; got {got}"
    )
}

pub(super) fn nested_composite_message(rel: &str, label: &str) -> String {
    format!(
        "{rel}: composite step \"{label}\" nests a configured action, where no caller-side timeout-minutes is expressible; call it from the workflow instead"
    )
}

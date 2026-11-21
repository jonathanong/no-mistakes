use super::lines;
use serde_yaml::Value;
use std::collections::{BTreeMap, BTreeSet};

pub(super) struct IgnoreEntry {
    pub(super) line: usize,
    pub(super) pattern: String,
    pub(super) case_insensitive: bool,
    pub(super) invalid_shape: bool,
}

pub(super) fn rule_directories(value: &Value) -> Vec<String> {
    match value.get("ruleDirs") {
        Some(Value::String(dir)) => vec![dir.clone()],
        Some(Value::Sequence(items)) => items
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect(),
        _ => Vec::new(),
    }
}

pub(super) fn ignores_all(documents: &[Value], source: &str) -> Vec<IgnoreEntry> {
    let mut used = BTreeSet::new();
    let mut seen = BTreeMap::new();
    let mut entries = Vec::new();
    for value in documents {
        match value {
            Value::Sequence(items) => {
                for item in items {
                    collect_rule(item, source, &mut used, &mut seen, &mut entries);
                }
            }
            Value::Mapping(_) => collect_rule(value, source, &mut used, &mut seen, &mut entries),
            _ => {}
        }
    }
    entries
}

fn collect_rule(
    value: &Value,
    source: &str,
    used: &mut BTreeSet<usize>,
    seen: &mut BTreeMap<String, usize>,
    out: &mut Vec<IgnoreEntry>,
) {
    let Some(ignores) = value.get("ignores") else {
        return;
    };
    let Some(items) = ignores.as_sequence() else {
        out.push(invalid(
            lines::take_line(source, "ignores:", used),
            String::new(),
            false,
        ));
        return;
    };
    for item in items {
        out.push(entry(item, source, used, seen));
    }
}

fn entry(
    item: &Value,
    source: &str,
    used: &mut BTreeSet<usize>,
    seen: &mut BTreeMap<String, usize>,
) -> IgnoreEntry {
    match item {
        Value::String(pattern) => IgnoreEntry {
            line: lines::take_pattern_line(source, pattern, seen, used),
            pattern: pattern.clone(),
            case_insensitive: false,
            invalid_shape: false,
        },
        Value::Mapping(_) => mapping_entry(item, source, used, seen),
        Value::Bool(flag) => {
            let needle = if *flag { "- true" } else { "- false" };
            invalid(lines::take_line(source, needle, used), String::new(), false)
        }
        _ => {
            let needle = serde_yaml::to_string(item).unwrap_or_default();
            invalid(
                lines::take_line(source, needle.trim(), used),
                String::new(),
                false,
            )
        }
    }
}

fn mapping_entry(
    item: &Value,
    source: &str,
    used: &mut BTreeSet<usize>,
    seen: &mut BTreeMap<String, usize>,
) -> IgnoreEntry {
    let pattern = item
        .get("glob")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let case_insensitive = item
        .get("caseInsensitive")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    if pattern.is_empty() {
        return invalid(
            lines::take_line(source, "glob:", used),
            pattern,
            case_insensitive,
        );
    }
    IgnoreEntry {
        line: lines::take_pattern_line(source, &pattern, seen, used),
        pattern,
        case_insensitive,
        invalid_shape: false,
    }
}

fn invalid(line: usize, pattern: String, case_insensitive: bool) -> IgnoreEntry {
    IgnoreEntry {
        line,
        pattern,
        case_insensitive,
        invalid_shape: true,
    }
}

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
#[path = "entries_tests.rs"]
mod tests;

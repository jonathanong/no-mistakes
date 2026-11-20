use serde_yaml::Value;
use std::collections::BTreeSet;

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

pub(super) fn ignores(value: &Value, source: &str) -> Vec<IgnoreEntry> {
    let mut used = BTreeSet::new();
    let mut entries = Vec::new();
    match value {
        Value::Sequence(items) => {
            for item in items {
                collect_rule(item, source, &mut used, &mut entries);
            }
        }
        Value::Mapping(_) => collect_rule(value, source, &mut used, &mut entries),
        _ => {}
    }
    entries
}

fn collect_rule(
    value: &Value,
    source: &str,
    used: &mut BTreeSet<usize>,
    out: &mut Vec<IgnoreEntry>,
) {
    let Some(ignores) = value.get("ignores") else {
        return;
    };
    let Some(items) = ignores.as_sequence() else {
        out.push(invalid(
            take_line(source, "ignores:", used),
            String::new(),
            false,
        ));
        return;
    };
    for item in items {
        out.push(entry(item, source, used));
    }
}

fn entry(item: &Value, source: &str, used: &mut BTreeSet<usize>) -> IgnoreEntry {
    match item {
        Value::String(pattern) => IgnoreEntry {
            line: take_pattern_line(source, pattern, used),
            pattern: pattern.clone(),
            case_insensitive: false,
            invalid_shape: false,
        },
        Value::Mapping(_) => mapping_entry(item, source, used),
        Value::Bool(flag) => {
            let needle = if *flag { "- true" } else { "- false" };
            invalid(take_line(source, needle, used), String::new(), false)
        }
        _ => {
            let needle = serde_yaml::to_string(item).unwrap_or_default();
            invalid(take_line(source, needle.trim(), used), String::new(), false)
        }
    }
}

fn mapping_entry(item: &Value, source: &str, used: &mut BTreeSet<usize>) -> IgnoreEntry {
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
        return invalid(take_line(source, "glob:", used), pattern, case_insensitive);
    }
    IgnoreEntry {
        line: take_pattern_line(source, &pattern, used),
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

fn take_pattern_line(source: &str, pattern: &str, used: &mut BTreeSet<usize>) -> usize {
    take_line_if(source, used, |line| line_has_pattern(line, pattern))
}

fn take_line(source: &str, needle: &str, used: &mut BTreeSet<usize>) -> usize {
    take_line_if(source, used, |line| line.contains(needle))
}

fn take_line_if(source: &str, used: &mut BTreeSet<usize>, matches: impl Fn(&str) -> bool) -> usize {
    source
        .lines()
        .enumerate()
        .find_map(|(index, line)| {
            let line_no = index + 1;
            (!used.contains(&line_no) && matches(line)).then(|| {
                used.insert(line_no);
                line_no
            })
        })
        .unwrap_or(1)
}

fn line_has_pattern(line: &str, pattern: &str) -> bool {
    let trimmed = line.trim().trim_start_matches("- ").trim();
    let quoted = [
        pattern.to_string(),
        format!("'{pattern}'"),
        format!("\"{pattern}\""),
    ];
    if quoted.iter().any(|item| trimmed == item) {
        return true;
    }
    trimmed
        .strip_prefix("glob:")
        .is_some_and(|rest| quoted.iter().any(|item| rest.trim() == item))
}

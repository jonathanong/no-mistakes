pub(super) fn yaml_has_key(value: &serde_yaml::Value, key: &str) -> bool {
    value
        .as_mapping()
        .is_some_and(|mapping| mapping.contains_key(serde_yaml::Value::String(key.to_string())))
}

pub(super) fn yaml_top_level_key_line(source: &str, key: &str) -> usize {
    let bare = format!("{key}:");
    let quoted = format!("\"{key}\":");
    source
        .lines()
        .position(|line| {
            if line.starts_with(' ') || line.starts_with('\t') {
                return false;
            }
            let trimmed = line.split('#').next().unwrap_or(line).trim();
            trimmed == bare || trimmed.starts_with(&bare) || trimmed.starts_with(&quoted)
        })
        .map(|index| index + 1)
        .unwrap_or(1)
}

pub(super) fn json_key_line(source: &str, key: &str) -> usize {
    json_quoted_key_after(source, &format!("\"{key}\""), 0).unwrap_or(1)
}

pub(super) fn json_nested_key_line(source: &str, parent: &str, key: &str) -> usize {
    let parent_needle = format!("\"{parent}\"");
    let Some(start) = source.find(&parent_needle) else {
        return 1;
    };
    json_quoted_key_after(source, &format!("\"{key}\""), start + parent_needle.len()).unwrap_or(1)
}

pub(super) fn json_quoted_key_after(source: &str, needle: &str, from: usize) -> Option<usize> {
    let rest = source.get(from..)?;
    let rel = rest.find(needle)?;
    Some(
        source[..from + rel]
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count()
            + 1,
    )
}

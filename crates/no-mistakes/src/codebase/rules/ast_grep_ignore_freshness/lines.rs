use std::collections::{BTreeMap, BTreeSet};

pub(super) fn take_line(source: &str, needle: &str, used: &mut BTreeSet<usize>) -> usize {
    take_unused(source, used, |line| line.contains(needle))
}

pub(super) fn take_pattern_line(
    source: &str,
    pattern: &str,
    seen: &mut BTreeMap<String, usize>,
    used: &mut BTreeSet<usize>,
) -> usize {
    let want = seen.get(pattern).copied().unwrap_or(0);
    let mut passed = 0usize;
    let mut block = None;
    for (index, line) in source.lines().enumerate() {
        let Some(span) = ignore_span(line, &mut block) else {
            continue;
        };
        let count = pattern_occurrences(span, pattern);
        if count == 0 {
            continue;
        }
        if passed + count > want {
            let line_no = index + 1;
            seen.insert(pattern.to_string(), want + 1);
            used.insert(line_no);
            return line_no;
        }
        passed += count;
    }
    seen.insert(pattern.to_string(), want + 1);
    used.insert(1);
    1
}

fn take_unused(source: &str, used: &mut BTreeSet<usize>, matches: impl Fn(&str) -> bool) -> usize {
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

fn ignore_span<'a>(line: &'a str, block: &mut Option<usize>) -> Option<&'a str> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return None;
    }
    if trimmed == "---" || trimmed.starts_with("--- ") {
        *block = None;
        return None;
    }
    let indent = line.len() - line.trim_start().len();
    if let Some(key_indent) = *block {
        if indent <= key_indent {
            *block = None;
        } else {
            return Some(line);
        }
    }
    let rest = trimmed.strip_prefix("ignores:")?;
    if rest.contains('[') {
        return Some(rest);
    }
    if rest.trim().is_empty() {
        *block = Some(indent);
    }
    None
}

fn pattern_occurrences(line: &str, pattern: &str) -> usize {
    if pattern.is_empty() {
        return 0;
    }
    let mut count = 0usize;
    let mut index = 0usize;
    while index < line.len() {
        if let Some(len) = match_at(line, index, pattern) {
            count += 1;
            index += len;
            continue;
        }
        index += line[index..]
            .chars()
            .next()
            .map(char::len_utf8)
            .unwrap_or(1);
    }
    count
}

fn match_at(line: &str, index: usize, pattern: &str) -> Option<usize> {
    let rest = line.get(index..)?;
    if rest.starts_with(['\'', '"']) {
        let quote = rest.as_bytes()[0];
        let quoted_len = pattern.len() + 2;
        if rest.len() >= quoted_len
            && rest.as_bytes()[quoted_len - 1] == quote
            && &rest[1..quoted_len - 1] == pattern
            && bounded(line, index, index + quoted_len)
        {
            return Some(quoted_len);
        }
        return None;
    }
    if rest.starts_with(pattern) && bounded(line, index, index + pattern.len()) {
        Some(pattern.len())
    } else {
        None
    }
}

fn bounded(line: &str, start: usize, end: usize) -> bool {
    let before = start == 0 || is_boundary(line.as_bytes()[start - 1]);
    let after = end >= line.len() || is_boundary(line.as_bytes()[end]);
    before && after
}

fn is_boundary(byte: u8) -> bool {
    matches!(
        byte,
        b'[' | b']' | b',' | b' ' | b'\t' | b':' | b'{' | b'}' | b'\r'
    )
}

#[cfg(test)]
#[path = "lines_tests.rs"]
mod tests;

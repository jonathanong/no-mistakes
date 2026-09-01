use super::{CompiledOptions, CompiledPattern, RuleFinding, RULE_ID};
use assertion_ranges::{assertion_start, is_code, source_ranges, SourceRanges};

mod assertion_ranges;

pub(super) fn check_source(file: &str, content: &str, opts: &CompiledOptions) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    let ranges = if opts.patterns.iter().any(|pattern| pattern.multiline) {
        source_ranges(content)
    } else {
        SourceRanges::default()
    };
    for pattern in &opts.patterns {
        if pattern.multiline {
            for matched in pattern.regex.find_iter(content) {
                if !has_matching_version_delimiters(matched.as_str()) {
                    continue;
                }
                if pattern.reason == "package.json dependency assertion"
                    && (!has_matching_raw_entry_delimiters(matched.as_str())
                        || is_version_field_assertion(matched.as_str()))
                {
                    continue;
                }
                if !is_code(&ranges.non_code, matched.start()) {
                    continue;
                }
                if !has_code_matcher(matched.as_str(), matched.start(), &ranges.non_code) {
                    continue;
                }
                let Some(start) = assertion_start(&ranges.assertions, matched.start()) else {
                    continue;
                };
                let line = line_at(content, start);
                let normalized = matched
                    .as_str()
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ");
                findings.push(finding(file, line, pattern, &normalized));
            }
        } else {
            scan_lines(file, content, pattern, &mut findings);
        }
    }
    findings
}

fn has_matching_version_delimiters(matched: &str) -> bool {
    let bytes = matched.as_bytes();
    let Some(&closing) = bytes.last() else {
        return false;
    };
    bytes[..bytes.len() - 1]
        .iter()
        .rfind(|byte| matches!(byte, b'\'' | b'"' | b'`'))
        .is_some_and(|opening| *opening == closing)
}

fn has_matching_raw_entry_delimiters(matched: &str) -> bool {
    let mut quotes = matched.bytes().rev().filter(|byte| is_quote(*byte));
    let (Some(value_close), Some(value_open), Some(key_close), Some(key_open)) =
        (quotes.next(), quotes.next(), quotes.next(), quotes.next())
    else {
        return false;
    };
    value_open == value_close && key_open == key_close
}

fn is_quote(byte: u8) -> bool {
    matches!(byte, b'\'' | b'"' | b'`')
}

fn is_version_field_assertion(matched: &str) -> bool {
    matched.contains(r#""version""#)
        || matched.contains(r#"\"version\""#)
        || matched.contains("'version'")
        || matched.contains(r#"\'version\'"#)
}

fn has_code_matcher(matched: &str, start: usize, non_code_ranges: &[(usize, usize)]) -> bool {
    let matcher_offset = [
        ".toBe(",
        ".toContain(",
        ".toEqual(",
        ".toStrictEqual(",
        ".toHaveProperty(",
    ]
    .iter()
    .flat_map(|token| matched.match_indices(token).map(|(offset, _)| offset))
    .max();

    matcher_offset.is_some_and(|offset| is_code(non_code_ranges, start + offset))
}

fn scan_lines(
    file: &str,
    content: &str,
    pattern: &CompiledPattern,
    findings: &mut Vec<RuleFinding>,
) {
    for (index, line) in content.lines().enumerate() {
        for matched in pattern.regex.find_iter(line) {
            if pattern.reject_preceding_at
                && matched.start() > 0
                && line.as_bytes()[matched.start() - 1] == b'@'
            {
                continue;
            }
            findings.push(finding(file, index + 1, pattern, matched.as_str()));
        }
    }
}

fn line_at(content: &str, offset: usize) -> usize {
    content[..offset]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count()
        + 1
}

fn finding(file: &str, line: usize, pattern: &CompiledPattern, matched: &str) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line,
        message: message(file, line, &pattern.reason, matched),
        import: Some(matched.to_string()),
        target: Some(pattern.reason.clone()),
    }
}

pub(super) fn message(file: &str, line: usize, reason: &str, matched: &str) -> String {
    format!("{file}:{line}: tests must not pin exact dependency versions ({reason}): `{matched}`")
}

#[cfg(test)]
mod tests;

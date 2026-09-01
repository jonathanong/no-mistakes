use super::{CompiledOptions, CompiledPattern, RuleFinding, RULE_ID};
use assertion_ranges::{assertion_start, is_code, source_ranges, SourceRanges};
use delimiters::{has_matching_raw_entry_delimiters, has_matching_version_delimiters};
use raw_literal_arg::{
    is_direct_argument as raw_literal_is_direct_argument, is_version_field_assertion,
};
use regex::{Match, Regex};
use std::sync::LazyLock;

mod assertion_ranges;
mod delimiters;
mod jsx_text_ranges;
mod raw_literal_arg;
mod received_prefix;

pub(super) fn check_source(file: &str, content: &str, opts: &CompiledOptions) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    let ranges = if opts.patterns.iter().any(|pattern| pattern.multiline) {
        let jsx_text_ranges = jsx_text_ranges::collect(file, content);
        let lexical_source = jsx_text_ranges::mask(content, &jsx_text_ranges);
        let mut ranges = source_ranges(&lexical_source);
        ranges.non_code.extend(jsx_text_ranges.iter().copied());
        merge_ranges(&mut ranges.non_code);
        ranges
    } else {
        SourceRanges::default()
    };
    for pattern in &opts.patterns {
        if pattern.multiline {
            for captures in pattern.regex.captures_iter(content) {
                let matched = captures.get(0).expect("full regex match");
                let raw_assertion = pattern.reason == "package.json dependency assertion";
                let raw_entry = if raw_assertion {
                    raw_manifest_entry(content, matched, &ranges.non_code)
                } else {
                    Some((matched.as_str(), None))
                };
                let Some((displayed, raw_literal_range)) = raw_entry else {
                    continue;
                };
                if raw_literal_range.is_some_and(|range| {
                    !raw_literal_is_direct_argument(content, matched, range, &ranges.non_code)
                }) {
                    continue;
                }
                let reversed_assertion = !raw_assertion
                    && displayed
                        .as_bytes()
                        .first()
                        .is_some_and(|byte| matches!(*byte, b'\'' | b'"' | b'`'));
                let version_literal = captures
                    .name("version")
                    .map_or(displayed, |version| version.as_str());
                if !has_matching_version_delimiters(version_literal, raw_assertion) {
                    continue;
                }
                if pattern.reason == "package.json dependency assertion"
                    && !has_matching_raw_entry_delimiters(displayed)
                {
                    continue;
                }
                if !reversed_assertion && !is_code(&ranges.non_code, matched.start()) {
                    continue;
                }
                if !has_code_matcher(matched.as_str(), matched.start(), &ranges.non_code) {
                    continue;
                }
                let Some(start) = assertion_start(&ranges.assertions, matched.start()) else {
                    continue;
                };
                if !raw_assertion
                    && !reversed_assertion
                    && !matched.as_str().starts_with("expect.poll")
                    && !received_prefix::is_transparent(
                        content,
                        start,
                        matched.start(),
                        &ranges.non_code,
                    )
                {
                    continue;
                }
                let line = line_at(content, start);
                let normalized = displayed.split_whitespace().collect::<Vec<_>>().join(" ");
                findings.push(finding(file, line, pattern, &normalized));
            }
        } else {
            scan_lines(file, content, pattern, &mut findings);
        }
    }
    findings
}

fn merge_ranges(ranges: &mut Vec<(usize, usize)>) {
    ranges.sort_unstable_by_key(|(start, _)| *start);
    let mut merged: Vec<(usize, usize)> = Vec::with_capacity(ranges.len());
    for &(start, end) in ranges.iter() {
        if let Some((_, previous_end)) = merged.last_mut() {
            if start <= *previous_end {
                *previous_end = (*previous_end).max(end);
                continue;
            }
        }
        merged.push((start, end));
    }
    *ranges = merged;
}

fn raw_manifest_entry<'a>(
    content: &'a str,
    matched: Match<'a>,
    non_code_ranges: &[(usize, usize)],
) -> Option<(&'a str, Option<(usize, usize)>)> {
    static ENTRY: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(
            r#"\\?["'][@A-Za-z0-9_./-]+\\?["']\s*:\s*\\?["'][~^]?\d+(?:\.\d+){0,2}(?:-[A-Za-z0-9-]+(?:\.[A-Za-z0-9-]+)*)?(?:\+[A-Za-z0-9-]+(?:\.[A-Za-z0-9-]+)*)?\\?["']"#,
        )
        .expect("raw package entry regex")
    });
    let offset = matched.end().saturating_sub(1);
    let &(start, end) = non_code_ranges
        .iter()
        .find(|(start, end)| *start <= offset && offset < *end)?;
    ENTRY
        .find_iter(&content[start..end])
        .find(|entry| {
            let displayed = entry.as_str();
            let continues_value = content
                .as_bytes()
                .get(start + entry.end())
                .is_some_and(|byte| {
                    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'+' | b'-')
                });
            has_matching_raw_entry_delimiters(displayed)
                && !is_version_field_assertion(displayed)
                && !continues_value
        })
        .map(|entry| (entry.as_str(), Some((start, end))))
}

fn has_code_matcher(matched: &str, start: usize, non_code_ranges: &[(usize, usize)]) -> bool {
    [
        "toBe",
        "toContain",
        "toEqual",
        "toStrictEqual",
        "toHaveProperty",
    ]
    .iter()
    .flat_map(|token| matched.match_indices(token).map(|(offset, _)| offset))
    .any(|offset| is_code(non_code_ranges, start + offset))
}

fn scan_lines(
    file: &str,
    content: &str,
    pattern: &CompiledPattern,
    findings: &mut Vec<RuleFinding>,
) {
    for (index, line_with_ending) in content.split_inclusive('\n').enumerate() {
        let line = line_with_ending.trim_end_matches(['\r', '\n']);
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

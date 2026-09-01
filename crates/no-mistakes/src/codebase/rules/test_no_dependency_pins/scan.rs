use super::{CompiledOptions, CompiledPattern, RuleFinding, RULE_ID};
use assertion_ranges::{assertion_ranges, assertion_start};

mod assertion_ranges;

pub(super) fn check_source(file: &str, content: &str, opts: &CompiledOptions) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    let assertion_ranges = if opts.patterns.iter().any(|pattern| pattern.multiline) {
        assertion_ranges(content)
    } else {
        Vec::new()
    };
    for pattern in &opts.patterns {
        if pattern.multiline {
            for matched in pattern.regex.find_iter(content) {
                let start = assertion_start(&assertion_ranges, matched.start());
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

use super::{CompiledOptions, CompiledPattern, RuleFinding, RULE_ID};

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

fn assertion_ranges(content: &str) -> Vec<(usize, usize)> {
    let bytes = content.as_bytes();
    let mut stack = Vec::new();
    let mut ranges = Vec::new();
    let mut quote = None::<u8>;
    let mut escaped = false;
    let mut line_comment = false;
    let mut block_comment = false;
    let mut index = 0;
    while index < bytes.len() {
        let byte = bytes[index];
        let next = bytes.get(index + 1).copied();
        if line_comment {
            line_comment = byte != b'\n';
            index += 1;
            continue;
        }
        if block_comment {
            if byte == b'*' && next == Some(b'/') {
                block_comment = false;
                index += 2;
            } else {
                index += 1;
            }
            continue;
        }
        if let Some(delimiter) = quote {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == delimiter {
                quote = None;
            }
            index += 1;
            continue;
        }
        match byte {
            b'/' if next == Some(b'/') => {
                line_comment = true;
                index += 2;
                continue;
            }
            b'/' if next == Some(b'*') => {
                block_comment = true;
                index += 2;
                continue;
            }
            b'\'' | b'"' | b'`' => quote = Some(byte),
            b'(' => stack.push(expect_token_start(bytes, index)),
            b')' => {
                if let Some(Some(start)) = stack.pop() {
                    ranges.push((start, index));
                }
            }
            _ => {}
        }
        index += 1;
    }
    ranges.extend(
        stack
            .into_iter()
            .flatten()
            .map(|start| (start, bytes.len())),
    );
    ranges.sort_unstable_by_key(|(start, _)| *start);
    ranges
}

fn assertion_start(ranges: &[(usize, usize)], match_start: usize) -> usize {
    let upper = ranges.partition_point(|(start, _)| *start <= match_start);
    upper
        .checked_sub(1)
        .and_then(|index| {
            let (start, end) = ranges[index];
            (match_start <= end).then_some(start)
        })
        .unwrap_or(match_start)
}

fn expect_token_start(bytes: &[u8], open_paren: usize) -> Option<usize> {
    let mut end = open_paren;
    while end > 0 && bytes[end - 1].is_ascii_whitespace() {
        end -= 1;
    }
    let start = end.checked_sub("expect".len())?;
    if &bytes[start..end] != b"expect" {
        return None;
    }
    if start > 0 && (bytes[start - 1].is_ascii_alphanumeric() || bytes[start - 1] == b'_') {
        return None;
    }
    Some(start)
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

use super::CspPolicy;

pub(super) fn parse_policies_from_text(text: &str) -> Vec<CspPolicy> {
    let mut policies = Vec::new();
    let mut block = CspPolicy::default();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("//") {
            flush_block(&mut policies, &mut block);
            continue;
        }
        let quoted = extract_quoted_fragments(trimmed);
        if !quoted.is_empty() {
            flush_block(&mut policies, &mut block);
            for fragment in quoted {
                policies.extend(CspPolicy::parse_list(fragment));
            }
            continue;
        }
        if let Some(value) = strip_header_prefix(trimmed) {
            flush_block(&mut policies, &mut block);
            policies.extend(CspPolicy::parse_list(value));
            continue;
        }
        if trimmed.contains(',') {
            let parsed = CspPolicy::parse_list(trimmed);
            if parsed.len() > 1 {
                flush_block(&mut policies, &mut block);
                policies.extend(parsed);
                continue;
            }
        }
        block.parse_single_policy(trimmed);
    }
    flush_block(&mut policies, &mut block);
    policies
}

fn flush_block(policies: &mut Vec<CspPolicy>, block: &mut CspPolicy) {
    if !block.is_empty() {
        policies.push(std::mem::take(block));
    }
}

fn extract_quoted_fragments(line: &str) -> Vec<&str> {
    let mut fragments = extract_delimited(line, '"', '"');
    if fragments.is_empty() {
        fragments = extract_single_quoted_attrs(line);
    }
    fragments
}

fn extract_delimited(mut rest: &str, open: char, close: char) -> Vec<&str> {
    let mut fragments = Vec::new();
    while let Some(idx) = rest.find(open) {
        rest = &rest[idx + open.len_utf8()..];
        let Some(end) = rest.find(close) else {
            break;
        };
        fragments.push(&rest[..end]);
        rest = &rest[end + close.len_utf8()..];
    }
    fragments
}

fn extract_single_quoted_attrs(mut rest: &str) -> Vec<&str> {
    let mut fragments = Vec::new();
    while let Some(idx) = rest.find("='") {
        rest = &rest[idx + 2..];
        let Some(end) = rest.find('\'') else {
            break;
        };
        fragments.push(&rest[..end]);
        rest = &rest[end + 1..];
    }
    fragments
}

fn strip_header_prefix(line: &str) -> Option<&str> {
    let (prefix, rest) = line.split_once(':')?;
    if prefix.is_empty()
        || prefix.chars().any(char::is_whitespace)
        || matches!(
            prefix.to_ascii_lowercase().as_str(),
            "http" | "https" | "ws" | "wss"
        )
        || rest.starts_with("//")
    {
        return None;
    }
    Some(rest.trim())
}

use std::collections::BTreeSet;

#[path = "folded.rs"]
mod folded;

/// Bind `value` to a real `uses` occurrence that the line scan has not already
/// judged. A later step, name, or comment that merely contains the text is not
/// an occurrence. A second real `uses` of the same text still is.
pub(super) fn claim_uses_line(
    lines: &[&str],
    claimed: &mut BTreeSet<usize>,
    value: &str,
) -> Option<usize> {
    for (index, line) in lines.iter().enumerate() {
        let line_num = index + 1;
        if claimed.contains(&line_num) {
            continue;
        }
        let matched = line_uses_scalar(line, value)
            || (index > 0 && bare_uses_key(lines[index - 1]) && scalar_line_equals(line, value));
        if matched {
            claimed.insert(line_num);
            return Some(line_num);
        }
    }
    folded::claim_folded_uses(lines, claimed, value)
}

fn bare_uses_key(line: &str) -> bool {
    let trimmed = line.trim();
    let trimmed = trimmed.strip_prefix('-').map(str::trim).unwrap_or(trimmed);
    matches!(trimmed, "uses:" | "\"uses\":" | "'uses':")
}

fn scalar_line_equals(line: &str, value: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with(['-', '#']) {
        return false;
    }
    let mut quote = None;
    let mut end = trimmed.len();
    for (index, ch) in trimmed.char_indices() {
        if let Some(active) = quote {
            if ch == active {
                quote = None;
            }
            continue;
        }
        match ch {
            '"' | '\'' => quote = Some(ch),
            '#' => {
                end = index;
                break;
            }
            _ => {}
        }
    }
    unquote(trimmed[..end].trim()) == value
}

fn unquote(scalar: &str) -> &str {
    let bytes = scalar.as_bytes();
    if bytes.len() >= 2 {
        let open = bytes[0];
        let close = bytes[bytes.len() - 1];
        if (open == b'"' || open == b'\'') && open == close {
            return &scalar[1..scalar.len() - 1];
        }
    }
    scalar
}

/// True when an unquoted `uses` key on this line has scalar `value`.
fn line_uses_scalar(line: &str, value: &str) -> bool {
    let mut quote: Option<char> = None;
    let mut token = String::new();
    let mut scalar = String::new();
    let mut after_colon = false;
    let mut key_was_uses = false;
    let mut flow_depth = 0_i32;
    let mut skip_anchor = false;

    for ch in line.chars() {
        if let Some(active) = quote {
            if ch == active {
                quote = None;
                if after_colon && finish_scalar(&scalar, key_was_uses, value) {
                    return true;
                }
                if after_colon {
                    scalar.clear();
                    key_was_uses = false;
                    after_colon = false;
                }
            } else if after_colon {
                scalar.push(ch);
            } else {
                token.push(ch);
            }
            continue;
        }
        if matches!(ch, '"' | '\'') {
            quote = Some(ch);
            if !after_colon {
                token.clear();
            }
            continue;
        }
        if ch == '#' {
            break;
        }
        // `&anchor value` and `*alias` sit between `uses:` and the scalar.
        if skip_anchor {
            if ch.is_whitespace() {
                skip_anchor = false;
            }
            continue;
        }
        if after_colon && scalar.is_empty() && matches!(ch, '&' | '*') {
            skip_anchor = true;
            continue;
        }
        if matches!(ch, '{' | '[') {
            flow_depth += 1;
            token.clear();
            scalar.clear();
            after_colon = false;
            key_was_uses = false;
            continue;
        }
        if matches!(ch, '}' | ']' | ',') {
            if finish_scalar(&scalar, key_was_uses, value) {
                return true;
            }
            if ch != ',' {
                flow_depth -= 1;
            }
            token.clear();
            scalar.clear();
            after_colon = false;
            key_was_uses = false;
            continue;
        }
        if ch.is_whitespace() || (ch == '-' && !after_colon && flow_depth == 0 && token.is_empty())
        {
            if after_colon && !scalar.is_empty() {
                if finish_scalar(&scalar, key_was_uses, value) {
                    return true;
                }
                // A block value ends the line. Inside `{...}` another key may follow.
                if flow_depth == 0 {
                    return false;
                }
                scalar.clear();
                key_was_uses = false;
                after_colon = false;
            } else if !after_colon {
                token.clear();
            }
            continue;
        }
        if ch == ':' && !after_colon {
            key_was_uses = token == "uses";
            token.clear();
            scalar.clear();
            after_colon = true;
            continue;
        }
        if after_colon {
            scalar.push(ch);
        } else {
            token.push(ch);
        }
    }
    finish_scalar(&scalar, key_was_uses, value)
}

fn finish_scalar(scalar: &str, key_was_uses: bool, value: &str) -> bool {
    key_was_uses && scalar == value
}

pub(super) fn is_github_script(uses: &str) -> bool {
    let uses = strip_unquoted_comment(uses).trim();
    let uses = unquote(uses);
    let action = uses.split('@').next().unwrap_or(uses).trim();
    // GitHub treats owner/repo as case-insensitive. Nothing else is folded.
    action
        .trim_end_matches('/')
        .eq_ignore_ascii_case("actions/github-script")
}

pub(super) fn normalize_uses(value: &str) -> String {
    unquote(strip_unquoted_comment(value).trim())
        .trim()
        .to_string()
}

fn unquote(value: &str) -> &str {
    let bytes = value.as_bytes();
    if bytes.len() >= 2
        && ((bytes[0] == b'"' && bytes[bytes.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[bytes.len() - 1] == b'\''))
    {
        return &value[1..value.len() - 1];
    }
    value
}

pub(super) fn is_block_header(value: &str) -> bool {
    let value = strip_anchor(value.trim());
    let mut chars = value.chars();
    if !matches!(chars.next(), Some('|' | '>')) {
        return false;
    }
    let rest = chars.as_str().trim();
    let rest = rest
        .strip_prefix('+')
        .or_else(|| rest.strip_prefix('-'))
        .unwrap_or(rest);
    rest.chars().all(|ch| ch.is_ascii_digit())
}

fn strip_anchor(value: &str) -> &str {
    let Some(rest) = value.strip_prefix('&') else {
        return value;
    };
    rest.split_once(char::is_whitespace)
        .map(|(_, after)| after.trim())
        .unwrap_or(value)
}

pub(super) fn strip_unquoted_comment(value: &str) -> &str {
    let mut quote: Option<char> = None;
    for (index, ch) in value.char_indices() {
        if let Some(active) = quote {
            if ch == active {
                quote = None;
            }
            continue;
        }
        if ch == '"' || ch == '\'' {
            quote = Some(ch);
            continue;
        }
        if ch == '#' {
            return &value[..index];
        }
    }
    value
}

pub(super) fn block_continues(line: &str, key_indent: usize) -> bool {
    line.trim().is_empty() || leading_indent(line) > key_indent
}

pub(super) fn scan_block_line(literal: bool, line: &str) -> bool {
    literal || !line.trim_start().starts_with('#')
}

pub(super) fn leading_indent(line: &str) -> usize {
    line.bytes()
        .take_while(|byte| *byte == b' ' || *byte == b'\t')
        .count()
}

pub(super) fn ends_inside_quote(text: &str, mut inside: bool, quote: char) -> bool {
    let mut escaped = false;
    for ch in text.chars() {
        if !inside {
            if ch == quote {
                inside = true;
            }
            continue;
        }
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == quote {
            inside = false;
        }
    }
    inside
}

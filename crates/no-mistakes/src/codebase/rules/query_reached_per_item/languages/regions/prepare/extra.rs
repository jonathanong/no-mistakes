use super::{copy_block_comment, copy_escaped, copy_line};

/// Lifetimes and labels (`'static`, `'outer:`) are not character literals.
pub(super) fn neutralize_rust_quotes(source: &str) -> String {
    let chars: Vec<char> = source.chars().collect();
    let mut out = String::with_capacity(source.len());
    let mut index = 0;
    while index < chars.len() {
        if chars[index] == '\'' && is_lifetime_or_label(&chars, index) {
            out.push(' ');
            index += 1;
            continue;
        }
        out.push(chars[index]);
        index += 1;
    }
    out
}

fn is_lifetime_or_label(chars: &[char], index: usize) -> bool {
    if index > 0 {
        let prev = chars[index - 1];
        if prev.is_ascii_alphanumeric() || prev == '_' {
            return false;
        }
    }
    let Some(next) = chars.get(index + 1).copied() else {
        return false;
    };
    if next == '\\' || !(next.is_ascii_alphabetic() || next == '_') {
        return false;
    }
    let mut end = index + 2;
    while chars
        .get(end)
        .is_some_and(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
    {
        end += 1;
    }
    chars.get(end) != Some(&'\'')
}

/// Rust `match (...)` uses a reserved keyword; replace standalone `match` before `(`.
pub(super) fn neutralize_rust_match(masked: &str) -> String {
    let bytes = masked.as_bytes();
    let mut out = masked.as_bytes().to_vec();
    let mut index = 0;
    while index + 5 <= bytes.len() {
        if &bytes[index..index + 5] == b"match" {
            let before_ok = index == 0
                || !(bytes[index - 1].is_ascii_alphanumeric()
                    || matches!(bytes[index - 1], b'_' | b'.' | b':' | b'#'));
            let mut cursor = index + 5;
            while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
                cursor += 1;
            }
            if before_ok && bytes.get(cursor) == Some(&b'(') {
                out[index..index + 5].fill(b' ');
                index += 5;
                continue;
            }
        }
        index += 1;
    }
    String::from_utf8(out).unwrap_or_else(|_| masked.to_string())
}

/// C# verbatim strings (`@"..."`, `@$"..."`) do not use backslash escapes; `""` escapes `"`.
pub(super) fn neutralize_csharp_verbatim(source: &str) -> String {
    let chars: Vec<char> = source.chars().collect();
    let mut out = String::with_capacity(source.len());
    let mut index = 0;
    while index < chars.len() {
        let ch = chars[index];
        if ch == '/' && chars.get(index + 1) == Some(&'/') {
            index = copy_line(&chars, &mut out, index);
            continue;
        }
        if ch == '/' && chars.get(index + 1) == Some(&'*') {
            index = copy_block_comment(&chars, &mut out, index);
            continue;
        }
        if ch == '"' || ch == '\'' {
            index = copy_escaped(&chars, &mut out, index, ch);
            continue;
        }
        if ch == '@' && chars.get(index + 1) == Some(&'"') {
            out.push_str("  ");
            index = mask_verbatim(&chars, &mut out, index + 2);
            continue;
        }
        if ch == '@' && chars.get(index + 1) == Some(&'$') && chars.get(index + 2) == Some(&'"') {
            out.push_str("   ");
            index = mask_verbatim(&chars, &mut out, index + 3);
            continue;
        }
        out.push(ch);
        index += 1;
    }
    out
}

fn mask_verbatim(chars: &[char], out: &mut String, mut index: usize) -> usize {
    while index < chars.len() {
        let inner = chars[index];
        if inner == '"' {
            if chars.get(index + 1) == Some(&'"') {
                out.push_str("  ");
                index += 2;
                continue;
            }
            out.push(' ');
            return index + 1;
        }
        out.push(if inner == '\n' { '\n' } else { ' ' });
        index += 1;
    }
    index
}

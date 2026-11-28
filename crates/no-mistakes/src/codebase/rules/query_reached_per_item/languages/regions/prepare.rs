use crate::codebase::lang_frontends::{mask_strings, strip_comments_keep_strings};

pub(super) fn prepare(source: &str, extension: &str) -> String {
    let normalized = match extension {
        "py" => keep_floor_div(source),
        "rs" => neutralize_rust_quotes(source),
        "go" => neutralize_go_raw_strings(source),
        _ => source.to_string(),
    };
    mask_strings(&strip_comments_keep_strings(&normalized))
}

/// Lifetimes and labels (`'static`, `'outer:`) are not character literals.
fn neutralize_rust_quotes(source: &str) -> String {
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

/// Go raw strings are backticks. The shared strip does not treat them as strings.
fn neutralize_go_raw_strings(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut raw = false;
    for ch in source.chars() {
        if ch == '`' {
            out.push(' ');
            raw = !raw;
            continue;
        }
        if raw && ch != '\n' {
            out.push(' ');
        } else {
            out.push(ch);
        }
    }
    out
}

/// Python `//` is floor division. The shared comment strip treats `//` as a line comment.
fn keep_floor_div(source: &str) -> String {
    let chars: Vec<char> = source.chars().collect();
    let mut out = String::with_capacity(source.len());
    let mut index = 0;
    while index < chars.len() {
        let ch = chars[index];
        if ch == '"' || ch == '\'' {
            index = copy_quoted(&chars, &mut out, index, ch);
            continue;
        }
        if ch == '/' && chars.get(index + 1) == Some(&'/') {
            out.push(' ');
            out.push(' ');
            index += 2;
            continue;
        }
        out.push(ch);
        index += 1;
    }
    out
}

fn copy_quoted(chars: &[char], out: &mut String, start: usize, quote: char) -> usize {
    let triple = chars.get(start + 1) == Some(&quote) && chars.get(start + 2) == Some(&quote);
    let width = if triple { 3 } else { 1 };
    for _ in 0..width {
        out.push(quote);
    }
    let mut index = start + width;
    while index < chars.len() {
        if !triple && chars[index] == '\\' {
            out.push('\\');
            index += 1;
            if index < chars.len() {
                out.push(chars[index]);
                index += 1;
            }
            continue;
        }
        if triple && chars.get(index..index + 3) == Some(&[quote, quote, quote]) {
            out.push(quote);
            out.push(quote);
            out.push(quote);
            return index + 3;
        }
        out.push(chars[index]);
        if !triple && chars[index] == quote {
            return index + 1;
        }
        index += 1;
    }
    index
}

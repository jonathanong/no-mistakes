use crate::codebase::lang_frontends::{mask_strings, strip_comments_keep_strings};

pub(super) fn prepare(source: &str, python: bool) -> String {
    let comments = if python {
        strip_comments_keep_strings(&keep_floor_div(source))
    } else {
        strip_comments_keep_strings(source)
    };
    mask_strings(&comments)
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

use crate::codebase::lang_frontends::{mask_strings, strip_comments_keep_strings};

mod extra;

pub(super) fn prepare(source: &str, extension: &str) -> String {
    let normalized = match extension {
        "py" => keep_floor_div(source),
        "rs" => extra::neutralize_rust_quotes(source),
        "go" => neutralize_go_raw_strings(source),
        "cs" => extra::neutralize_csharp_verbatim(source),
        _ => source.to_string(),
    };
    let masked = mask_strings(&strip_comments_keep_strings(&normalized));
    if extension == "rs" {
        extra::neutralize_rust_match(&masked)
    } else {
        masked
    }
}

/// Go raw strings are backticks. The shared strip does not treat them as strings.
fn neutralize_go_raw_strings(source: &str) -> String {
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
        if ch == '`' {
            out.push(' ');
            index += 1;
            while index < chars.len() {
                let inner = chars[index];
                index += 1;
                if inner == '`' {
                    out.push(' ');
                    break;
                }
                out.push(if inner == '\n' { '\n' } else { ' ' });
            }
            continue;
        }
        out.push(ch);
        index += 1;
    }
    out
}

fn copy_line(chars: &[char], out: &mut String, mut index: usize) -> usize {
    while index < chars.len() && chars[index] != '\n' {
        out.push(chars[index]);
        index += 1;
    }
    index
}

fn copy_block_comment(chars: &[char], out: &mut String, mut index: usize) -> usize {
    out.push('/');
    out.push('*');
    index += 2;
    while index < chars.len() {
        out.push(chars[index]);
        if chars[index] == '*' && chars.get(index + 1) == Some(&'/') {
            out.push('/');
            return index + 2;
        }
        index += 1;
    }
    index
}

fn copy_escaped(chars: &[char], out: &mut String, start: usize, quote: char) -> usize {
    out.push(quote);
    let mut index = start + 1;
    while index < chars.len() {
        let ch = chars[index];
        out.push(ch);
        index += 1;
        if ch == '\\' && index < chars.len() {
            out.push(chars[index]);
            index += 1;
            continue;
        }
        if ch == quote {
            return index;
        }
    }
    index
}

/// Python `//` is floor division. The shared comment strip treats `//` as a line comment.
fn keep_floor_div(source: &str) -> String {
    let chars: Vec<char> = source.chars().collect();
    let mut out = String::with_capacity(source.len());
    let mut index = 0;
    while index < chars.len() {
        let ch = chars[index];
        if ch == '#' {
            index = copy_line(&chars, &mut out, index);
            continue;
        }
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

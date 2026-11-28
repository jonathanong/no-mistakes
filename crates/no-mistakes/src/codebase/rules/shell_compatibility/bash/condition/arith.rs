use super::super::super::lex::{assignment, command_words, identifier, Word};
use super::super::assign::single_word;

pub(super) fn scan_subscript_writes(
    words: &[Word<'_>],
    on_write: &mut dyn FnMut(&str),
    mut scan: impl FnMut(&str, &mut dyn FnMut(&str)),
) {
    let mut cmd = command_words(words).peekable();
    while let Some(word) = cmd.peek() {
        if let Some(sub) = assignment_subscript(word.raw) {
            scan(sub, on_write);
            cmd.next();
        } else if assignment(word.raw).is_some() {
            cmd.next();
        } else {
            break;
        }
    }
    let Some(first) = cmd.next() else {
        return;
    };
    if matches!(
        first.raw,
        "declare" | "typeset" | "local" | "readonly" | "export"
    ) {
        for word in cmd {
            let unquoted = single_word(word.raw);
            let raw = unquoted.as_deref().unwrap_or(word.raw);
            if let Some(sub) = assignment_subscript(raw) {
                scan(sub, on_write);
            }
        }
    } else if matches!(first.raw, "unset" | "read") {
        for word in cmd {
            if let Some(unquoted) = single_word(word.raw) {
                if let Some(sub) = indexed_subscript(&unquoted) {
                    scan(sub, on_write);
                }
            }
        }
    }
}

fn assignment_subscript(raw: &str) -> Option<&str> {
    let (lhs, _) = raw.split_once('=')?;
    indexed_subscript(lhs.strip_suffix('+').unwrap_or(lhs))
}

fn indexed_subscript(raw: &str) -> Option<&str> {
    let (name, sub) = raw.strip_suffix(']')?.split_once('[')?;
    (identifier(name) && !sub.is_empty()).then_some(sub)
}

pub(super) fn scan_inline_arith_writes(
    raw: &str,
    on_write: &mut dyn FnMut(&str),
    mut scan: impl FnMut(&str, &mut dyn FnMut(&str)),
) {
    let bytes = raw.as_bytes();
    let (mut i, mut double_quoted) = (0, false);
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i += 2,
            b'\'' if !double_quoted => {
                i += 1;
                while i < bytes.len() && bytes[i] != b'\'' {
                    i += 1;
                }
                i += 1;
            }
            b'"' => {
                double_quoted = !double_quoted;
                i += 1;
            }
            b'$' if bytes.get(i..i + 3) == Some(b"$((") => {
                let after = &raw[i + 3..];
                let Some(end) = matching_arith_end(after) else {
                    break;
                };
                scan(&after[..end], on_write);
                i += 3 + end + 2;
            }
            _ => i += 1,
        }
    }
}

fn matching_arith_end(body: &str) -> Option<usize> {
    let bytes = body.as_bytes();
    let mut depth = 0usize;
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'(' => depth += 1,
            b')' if depth == 0 => {
                return (bytes.get(index + 1) == Some(&b')')).then_some(index);
            }
            b')' => depth -= 1,
            _ => {}
        }
        index += 1;
    }
    None
}

pub(super) fn collect_substring_vars(raw: &str, mut i: usize, on_ident: &mut dyn FnMut(&str)) {
    let bytes = raw.as_bytes();
    if bytes.get(i) != Some(&b':')
        || matches!(bytes.get(i + 1), None | Some(b'-' | b'=' | b'+' | b'?'))
    {
        return;
    }
    i += 1;
    let start = i;
    let mut depth = 1usize;
    while i < bytes.len() && depth > 0 {
        if bytes[i] == b'{' {
            depth += 1;
        } else if bytes[i] == b'}' {
            depth -= 1;
        }
        if depth > 0 {
            i += 1;
        }
    }
    if depth != 0 {
        return;
    }
    let expr = &raw[start..i];
    let ebytes = expr.as_bytes();
    let mut j = 0;
    while j < ebytes.len() {
        if ebytes[j] == b'_' || ebytes[j].is_ascii_alphabetic() {
            let s = j;
            while j < ebytes.len() && (ebytes[j] == b'_' || ebytes[j].is_ascii_alphanumeric()) {
                j += 1;
            }
            on_ident(&expr[s..j]);
        } else {
            j += 1;
        }
    }
}

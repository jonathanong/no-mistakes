pub(super) fn keyword(source: &str) -> &str {
    source
        .split(|ch: char| ch.is_whitespace() || matches!(ch, ';' | '|' | '&'))
        .next()
        .unwrap_or_default()
}

pub(super) fn unsupported_control(source: &str, in_double_bracket: bool) -> bool {
    let source = source
        .trim_start_matches(|ch: char| ch.is_whitespace() || matches!(ch, '|' | '&' | '!'));
    let word = keyword(source);
    (!in_double_bracket && source.starts_with(['(', '{']))
        || matches!(
            word,
            "if" | "then"
                | "elif"
                | "else"
                | "fi"
                | "case"
                | "esac"
                | "while"
                | "until"
                | "for"
                | "do"
                | "done"
                | "select"
                | "function"
                | "time"
                | "coproc"
        )
        || word.ends_with("()")
}

enum Expansion {
    None,
    Simple(usize),
    Complex,
}

pub(super) fn has_complex_expansion(source: &str) -> bool {
    let bytes = source.as_bytes();
    let mut quote: Option<u8> = None;
    let mut i = 0;
    while i < bytes.len() {
        let byte = bytes[i];
        match quote {
            Some(b'\'') => {
                if byte == b'\'' {
                    quote = None;
                }
                i += 1;
            }
            Some(b'"') => {
                if byte == b'\\'
                    && bytes
                        .get(i + 1)
                        .is_some_and(|next| matches!(next, b'"' | b'\\' | b'$' | b'`' | b'\n'))
                {
                    i += 2;
                } else if byte == b'"' {
                    quote = None;
                    i += 1;
                } else {
                    match check_expansion(bytes, i) {
                        Expansion::Complex => return true,
                        Expansion::Simple(next) => i = next,
                        Expansion::None => i += 1,
                    }
                }
            }
            _ => {
                if byte == b'\\' {
                    i += 2;
                } else if matches!(byte, b'\'' | b'"') {
                    quote = Some(byte);
                    i += 1;
                } else if is_comment_start(bytes, i) {
                    i = skip_comment(bytes, i);
                } else {
                    match check_expansion(bytes, i) {
                        Expansion::Complex => return true,
                        Expansion::Simple(next) => i = next,
                        Expansion::None => i += 1,
                    }
                }
            }
        }
    }
    false
}

pub(super) fn has_arithmetic_command(source: &str) -> bool {
    let bytes = source.as_bytes();
    let mut quote: Option<u8> = None;
    let mut at_cmd_start = true;
    let mut i = 0;
    while i < bytes.len() {
        let byte = bytes[i];
        match quote {
            Some(b'\'') => {
                if byte == b'\'' {
                    quote = None;
                }
                i += 1;
            }
            Some(b'"') => {
                if byte == b'\\' && i + 1 < bytes.len() {
                    i += 2;
                } else {
                    if byte == b'"' {
                        quote = None;
                    }
                    i += 1;
                }
            }
            _ => {
                if byte == b'\\' {
                    at_cmd_start = false;
                    i += 2;
                } else if matches!(byte, b'\'' | b'"') {
                    at_cmd_start = false;
                    quote = Some(byte);
                    i += 1;
                } else if is_comment_start(bytes, i) {
                    i = skip_comment(bytes, i);
                } else if matches!(byte, b';' | b'\n' | b'|' | b'&') {
                    at_cmd_start = true;
                    i += 1;
                } else if byte.is_ascii_whitespace() {
                    i += 1;
                } else if at_cmd_start && byte == b'(' && bytes.get(i + 1) == Some(&b'(') {
                    return true;
                } else {
                    at_cmd_start = false;
                    i += 1;
                }
            }
        }
    }
    false
}

fn is_comment_start(bytes: &[u8], i: usize) -> bool {
    bytes[i] == b'#'
        && (i == 0
            || bytes[i - 1].is_ascii_whitespace()
            || matches!(bytes[i - 1], b';' | b'|' | b'&'))
}

fn skip_comment(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len() && bytes[i] != b'\n' {
        i += 1;
    }
    i
}

fn check_expansion(bytes: &[u8], i: usize) -> Expansion {
    let byte = bytes[i];
    if byte == b'`' {
        return Expansion::Complex;
    }
    if byte != b'$' {
        return Expansion::None;
    }
    match bytes.get(i + 1) {
        Some(b'(' | b'[') => Expansion::Complex,
        Some(b'{') => {
            let start = i + 2;
            let mut end = start;
            while end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
                end += 1;
            }
            if end > start && bytes.get(end) == Some(&b'}') {
                Expansion::Simple(end + 1)
            } else {
                Expansion::Complex
            }
        }
        _ => Expansion::None,
    }
}

use super::super::super::lex::{command_words, Word};
use crate::fx::FxHashSet;

fn arith_body(raw: &str) -> Option<&str> {
    raw.strip_prefix("((")?.strip_suffix("))")
}

fn is_arith_cmp(raw: &str) -> bool {
    matches!(raw, "-eq" | "-ne" | "-lt" | "-le" | "-gt" | "-ge")
}

pub(in super::super) fn collect_arithmetic_writes(
    words: &[Word<'_>],
    mut on_write: impl FnMut(&str),
) {
    let mut cmd = command_words(words);
    if let Some(first) = cmd.next() {
        if let Some(expr) = arith_body(first.raw) {
            scan_arith_writes(expr, &mut on_write);
        } else if first.raw == "for" {
            if let Some(expr) = cmd.next().and_then(|w| arith_body(w.raw)) {
                scan_arith_writes(expr, &mut on_write);
            }
        } else if first.raw == "[[" {
            let rest: Vec<&str> = cmd.map(|w| w.raw).collect();
            for (idx, raw) in rest.iter().enumerate() {
                let adj_op = (idx > 0 && is_arith_cmp(rest[idx - 1]))
                    || rest.get(idx + 1).is_some_and(|next| is_arith_cmp(next));
                if adj_op {
                    scan_arith_writes(raw, &mut on_write);
                }
            }
        } else if first.raw == "let" {
            for arg in cmd {
                let raw = arg
                    .raw
                    .strip_prefix('"')
                    .and_then(|s| s.strip_suffix('"'))
                    .or_else(|| arg.raw.strip_prefix('\'')?.strip_suffix('\''))
                    .unwrap_or(arg.raw);
                scan_arith_writes(raw, &mut on_write);
            }
        }
    }
    super::arith::scan_subscript_writes(words, &mut on_write, scan_arith_writes);
    for word in words {
        super::arith::scan_inline_arith_writes(word.raw, &mut on_write, scan_arith_writes);
    }
}

fn scan_arith_writes(expr: &str, on_write: &mut dyn FnMut(&str)) {
    let bytes = expr.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'_' || bytes[i].is_ascii_alphabetic() {
            let start = i;
            while i < bytes.len() && (bytes[i] == b'_' || bytes[i].is_ascii_alphanumeric()) {
                i += 1;
            }
            let ident = &expr[start..i];
            let mut after = i + bytes[i..]
                .iter()
                .take_while(|b| b.is_ascii_whitespace())
                .count();
            if bytes.get(after) == Some(&b'[') {
                after = skip_bracket(bytes, after);
                after += bytes[after..]
                    .iter()
                    .take_while(|b| b.is_ascii_whitespace())
                    .count();
            }
            if has_inc_dec_before(bytes, start) || has_arith_write_after(bytes, after) {
                on_write(ident);
            }
        } else {
            i += 1;
        }
    }
}

fn skip_bracket(bytes: &[u8], mut i: usize) -> usize {
    let mut depth = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'[' {
            depth += 1;
        } else if bytes[i] == b']' {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return i + 1;
            }
        }
        i += 1;
    }
    i
}

fn has_inc_dec_before(bytes: &[u8], start: usize) -> bool {
    let trimmed = bytes[..start].trim_ascii_end();
    trimmed.ends_with(b"++") || trimmed.ends_with(b"--")
}

fn has_arith_write_after(bytes: &[u8], i: usize) -> bool {
    let rest = &bytes[i..];
    rest.starts_with(b"++")
        || rest.starts_with(b"--")
        || (rest.starts_with(b"=") && !rest.starts_with(b"=="))
        || matches!(
            rest.get(..2),
            Some(b"+=" | b"-=" | b"*=" | b"/=" | b"%=" | b"&=" | b"^=" | b"|=")
        )
        || matches!(rest.get(..3), Some(b"<<=" | b">>="))
}

pub(super) fn collect_vars(words: &[&Word<'_>], vars: &mut FxHashSet<String>) {
    let in_cond = words.iter().any(|w| w.raw == "[[");
    let mut prev = "";
    for (idx, word) in words.iter().enumerate() {
        let raw = word.raw;
        if matches!(prev, "-v" | "-R") {
            collect_identifiers(raw, vars);
        }
        if let Some(expr) = arith_body(raw) {
            collect_identifiers(expr, vars);
        } else {
            collect_dollar_vars(raw, vars);
            if in_cond
                && (is_arith_cmp(prev) || words.get(idx + 1).is_some_and(|n| is_arith_cmp(n.raw)))
            {
                collect_identifiers(raw, vars);
            }
        }
        prev = raw;
    }
}

fn collect_identifiers(expr: &str, vars: &mut FxHashSet<String>) {
    let bytes = expr.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'_' || bytes[i].is_ascii_alphabetic() {
            let start = i;
            while i < bytes.len() && (bytes[i] == b'_' || bytes[i].is_ascii_alphanumeric()) {
                i += 1;
            }
            vars.insert(expr[start..i].to_owned());
        } else {
            i += 1;
        }
    }
}

fn collect_dollar_vars(raw: &str, vars: &mut FxHashSet<String>) {
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
            b'$' => {
                i += 1;
                let braced = bytes.get(i) == Some(&b'{');
                if braced {
                    i += 1 + usize::from(matches!(bytes.get(i + 1), Some(b'!' | b'#')));
                }
                if bytes
                    .get(i)
                    .is_some_and(|&b| b == b'_' || b.is_ascii_alphabetic())
                {
                    let start = i;
                    while bytes
                        .get(i)
                        .is_some_and(|&b| b == b'_' || b.is_ascii_alphanumeric())
                    {
                        i += 1;
                    }
                    vars.insert(raw[start..i].to_owned());
                    if braced {
                        super::arith::collect_substring_vars(raw, i, &mut |n| {
                            vars.insert(n.to_owned());
                        });
                    }
                }
            }
            _ => i += 1,
        }
    }
}

//! Whether a comma sits inside a YAML or JSON flow collection.
//!
//! `{ env: { A: 1 }, uses: ... }` has a comma that separates entries, while
//! `Homebrew [see note], uses: ...` is prose. The text is a single line of
//! test source, so the scan is a heuristic: it balances `{`/`[` against `}`/`]`
//! and skips quoted scalars (`{ name: "}", uses: ... }`).

/// True when `text` leaves a `{` or `[` open.
///
/// A quote opens a string only where a scalar starts: after `{`, `[`, `,`, or
/// `:`, with an optional `\` for an escaped quote (`\"`). Prose such as
/// `don't` is not an opener. A quote with no end is not a string: the text is
/// scanned again with that quote read as a plain character, so the quote of an
/// outer JavaScript string (`toEqual(['{ name: x`) does not hide the braces
/// inside it.
pub(super) fn inside_flow(text: &str) -> bool {
    let bytes = text.as_bytes();
    let mut literal_until = 0;
    loop {
        match open_depth(bytes, literal_until) {
            Ok(depth) => return depth > 0,
            Err(unterminated) => literal_until = unterminated + 1,
        }
    }
}

/// A string opened at `start`, and how many backslashes write each of its quotes:
/// none for `"`, one for `\"` (YAML inside one JavaScript string), three for
/// `\\\"` (inside two).
struct Span {
    start: usize,
    quote: u8,
    escapes: usize,
}

/// The bracket depth after the whole text, or the start of a string that never
/// ends. A quote before `literal_until` is never an opener.
fn open_depth(bytes: &[u8], literal_until: usize) -> Result<usize, usize> {
    let mut depth = 0_usize;
    let mut span: Option<Span> = None;
    let mut at = 0;
    while at < bytes.len() {
        let byte = bytes[at];
        if let Some(open) = &span {
            let (next, closed) = step_in_string(bytes, at, open);
            at = next;
            if closed {
                span = None;
            }
            continue;
        }
        let may_open = at >= literal_until && starts_scalar(bytes, at);
        match byte {
            b'{' | b'[' => depth += 1,
            b'}' | b']' => depth = depth.saturating_sub(1),
            b'"' | b'\'' | b'`' if may_open => {
                span = Some(Span {
                    start: at,
                    quote: byte,
                    escapes: 0,
                });
            }
            b'\\' if may_open => {
                if let Some((escapes, quote)) = escaped_opener(bytes, at) {
                    span = Some(Span {
                        start: at,
                        quote,
                        escapes,
                    });
                }
            }
            _ => {}
        }
        at += 1;
    }
    span.map_or(Ok(depth), |open| Err(open.start))
}

/// The backslash run at `at` and the quote it escapes, when the run writes a
/// quote one or more JavaScript string layers deep: `\"` is one layer and
/// `\\\"` is two (each layer doubles the backslashes and adds the one that
/// escapes the quote, so the run is one less than a power of two).
fn escaped_opener(bytes: &[u8], at: usize) -> Option<(usize, u8)> {
    let run = backslash_run(bytes, at);
    let quote = *bytes.get(at + run).filter(|&byte| is_quote(Some(byte)))?;
    (run + 1).is_power_of_two().then_some((run, quote))
}

fn backslash_run(bytes: &[u8], at: usize) -> usize {
    bytes[at..]
        .iter()
        .take_while(|&&byte| byte == b'\\')
        .count()
}

/// One step inside a string: the next position and whether the string ended.
/// A plain string ends at its quote and a backslash skips the next byte. In
/// single quotes a doubled `''` is a literal quote.
fn step_in_string(bytes: &[u8], at: usize, span: &Span) -> (usize, bool) {
    if span.escapes > 0 {
        return step_in_escaped_string(bytes, at, span);
    }
    let next = bytes.get(at + 1);
    let doubled_quote = bytes[at] == span.quote && span.quote == b'\'' && next == Some(&b'\'');
    if bytes[at] == b'\\' || doubled_quote {
        (at + 2, false)
    } else {
        (at + 1, bytes[at] == span.quote)
    }
}

/// One step inside a string whose quotes are written with backslashes. Every
/// JavaScript layer doubles the backslashes of the YAML text and writes a quote
/// with one more, so a run of `escapes + (escapes + 1) * k` backslashes before the
/// quote holds `k` YAML backslashes.
fn step_in_escaped_string(bytes: &[u8], at: usize, span: &Span) -> (usize, bool) {
    if bytes[at] != b'\\' {
        return (at + 1, false);
    }
    let run = backslash_run(bytes, at);
    let ends = bytes.get(at + run) == Some(&span.quote) && closes_scalar(run, span);
    (at + run + usize::from(ends), ends)
}

/// Whether a quote written after `run` backslashes ends the scalar. In double
/// quotes the quote ends it when `k` is even and is a quote of the scalar when
/// `k` is odd: `\"a\\\"}x\"` is one string holding `a"}x`, not a string that ends
/// at `\\\"`. In single quotes a backslash is literal, so any `k` ends it:
/// `\'a\\\'` is the scalar `a\`.
fn closes_scalar(run: usize, span: &Span) -> bool {
    let layer = span.escapes + 1;
    let Some(extra) = run.checked_sub(span.escapes) else {
        return false;
    };
    extra % layer == 0 && (span.quote == b'\'' || (extra / layer).is_multiple_of(2))
}

fn is_quote(byte: Option<&u8>) -> bool {
    matches!(byte, Some(b'"' | b'\'' | b'`'))
}

/// True when the previous non-space byte is where a flow scalar can start.
fn starts_scalar(bytes: &[u8], at: usize) -> bool {
    let before = bytes[..at].trim_ascii_end();
    matches!(before.last(), Some(b'{' | b'[' | b',' | b':'))
}

#[cfg(test)]
mod tests;

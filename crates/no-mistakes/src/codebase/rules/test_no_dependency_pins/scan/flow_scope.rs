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

/// A string opened at `start`, and whether its quotes are written `\"`.
struct Span {
    start: usize,
    quote: u8,
    escaped: bool,
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
                    escaped: false,
                });
            }
            b'\\' if may_open && is_quote(bytes.get(at + 1)) => {
                span = Some(Span {
                    start: at,
                    quote: bytes[at + 1],
                    escaped: true,
                });
            }
            _ => {}
        }
        at += 1;
    }
    span.map_or(Ok(depth), |open| Err(open.start))
}

/// One step inside a string: the next position and whether the string ended.
/// A plain string ends at its quote and a backslash skips the next byte; a
/// `\"` string ends at `\"`. In single quotes a doubled `''` is a literal quote.
fn step_in_string(bytes: &[u8], at: usize, span: &Span) -> (usize, bool) {
    let next = bytes.get(at + 1);
    if span.escaped {
        return if bytes[at] == b'\\' && next == Some(&span.quote) {
            (at + 2, true)
        } else {
            (at + 1, false)
        };
    }
    let doubled_quote = bytes[at] == span.quote && span.quote == b'\'' && next == Some(&b'\'');
    if bytes[at] == b'\\' || doubled_quote {
        (at + 2, false)
    } else {
        (at + 1, bytes[at] == span.quote)
    }
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

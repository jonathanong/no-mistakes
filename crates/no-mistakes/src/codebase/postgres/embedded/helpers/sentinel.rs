//! Private hole markers for imported-helper calls.
//!
//! The single-file walk cannot know what an imported helper returns, so it
//! composes `MARK <site> MARK` into the SQL text where the helper's text would
//! go. Every public fact is scrubbed of these markers before it leaves
//! extraction (see `finalize`); only the pending-call templates keep them.

use std::ops::Range;

/// A Unicode noncharacter: it never appears in real SQL and differs from the
/// internal placeholder's `U+10FFFF`, so placeholder counting and renumbering
/// ignore it.
pub(crate) const MARK: char = '\u{10FFFE}';

pub(crate) fn encode(site: usize) -> String {
    format!("{MARK}{site}{MARK}")
}

/// Byte range and site index of every well-formed marker, in text order.
pub(crate) fn find_all(text: &str) -> Vec<(Range<usize>, usize)> {
    let mut found = Vec::new();
    let mut at = 0;
    while let Some(relative) = text[at..].find(MARK) {
        let start = at + relative;
        let digits_start = start + MARK.len_utf8();
        let digits = text[digits_start..]
            .bytes()
            .take_while(u8::is_ascii_digit)
            .count();
        let end = digits_start + digits;
        let well_formed = digits > 0 && text[end..].starts_with(MARK);
        if !well_formed {
            at = digits_start;
            continue;
        }
        let close = end + MARK.len_utf8();
        if let Ok(site) = text[digits_start..end].parse() {
            found.push((start..close, site));
        }
        at = close;
    }
    found
}

pub(crate) fn contains(text: &str) -> bool {
    !find_all(text).is_empty()
}

/// Whether the text is nothing but holes: the result of a speculative
/// imported call, which is not yet known to be a SQL builder.
pub(crate) fn is_bare(text: &str) -> bool {
    let holes = find_all(text);
    let hole_bytes: usize = holes.iter().map(|(range, _)| range.len()).sum();
    !holes.is_empty() && hole_bytes == text.len()
}

/// Text before the first marker.
pub(crate) fn prefix(text: &str) -> &str {
    find_all(text)
        .first()
        .map_or(text, |(range, _)| &text[..range.start])
}

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

pub(crate) fn in_identifier_slot(text: &str, hole_start: usize) -> bool {
    identifier_slot(text, hole_start)
}

/// The two sides of a removed fragment splice would form one token.
pub(crate) fn fragment_separator(text: &str, start: usize, end: usize) -> bool {
    let word = |ch: char| ch.is_ascii_alphanumeric() || ch == '_';
    let next_word = text[end..].chars().next().is_some_and(word);
    let prev_space = text[..start]
        .chars()
        .next_back()
        .is_some_and(|ch| ch.is_whitespace());
    let prev_word = text[..start].chars().next_back().is_some_and(word);
    (next_word && !prev_space) || (prev_word && next_word)
}

/// A value hole: comparison or arithmetic operand, `LIMIT`/`OFFSET`,
/// `VALUES` list, or function argument. A column list after
/// `INTO`/`UPDATE`/`FROM`/`JOIN`/`TABLE` is not one.
pub(crate) fn value_position(text: &str) -> bool {
    let trimmed = text.trim_end();
    let Some(tail) = trimmed.chars().next_back() else {
        return false;
    };
    if matches!(tail, '=' | '<' | '>' | '+' | '-' | '*' | '/') {
        return true;
    }
    let word = last_ident(trimmed);
    if word.eq_ignore_ascii_case("limit") || word.eq_ignore_ascii_case("offset") {
        return true;
    }
    let Some(open) = trimmed.rfind('(') else {
        return false;
    };
    if trimmed[open + 1..].contains(')') {
        return false;
    }
    let inside = trimmed[open + 1..].trim_end();
    if !(inside.is_empty() || inside.ends_with(',')) {
        return false;
    }
    let before = trimmed[..open].trim_end();
    let word = last_ident(before);
    if word.eq_ignore_ascii_case("values") {
        return true;
    }
    let prev = ident_before(before, word);
    if ["into", "update", "from", "join", "table"]
        .iter()
        .any(|keyword| prev.eq_ignore_ascii_case(keyword))
    {
        return false;
    }
    !word.is_empty()
}

fn last_ident(text: &str) -> &str {
    let end = text.trim_end();
    let start = end
        .rfind(|c: char| !c.is_ascii_alphanumeric() && c != '_')
        .map(|at| at + end[at..].chars().next().map(char::len_utf8).unwrap_or(0))
        .unwrap_or(0);
    &end[start..]
}

fn ident_before<'a>(text: &'a str, word: &str) -> &'a str {
    let head = text.trim_end();
    last_ident(
        head.get(..head.len().saturating_sub(word.len()))
            .unwrap_or(""),
    )
}

fn identifier_slot(text: &str, hole_start: usize) -> bool {
    let before = &text[..hole_start];
    let Some(ch) = before.chars().next_back() else {
        return false;
    };
    let head = if ch.is_ascii_alphanumeric() || ch == '_' {
        before.trim_end_matches(|c: char| c.is_ascii_alphanumeric() || c == '_')
    } else if ch.is_whitespace() {
        before
    } else {
        return false;
    };
    let word = head
        .trim_end()
        .rsplit(|c: char| !c.is_ascii_alphanumeric() && c != '_')
        .find(|word| !word.is_empty())
        .unwrap_or("");
    ["select", "from", "join", "update", "into", "table"]
        .iter()
        .any(|keyword| word.eq_ignore_ascii_case(keyword))
}

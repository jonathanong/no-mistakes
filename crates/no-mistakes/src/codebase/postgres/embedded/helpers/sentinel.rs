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

/// A marker glued to an identifier character, as in `` tmp_${expr} ``.
#[cfg(test)]
pub(crate) fn splits_identifier(text: &str) -> bool {
    find_all(text).iter().any(|(range, _)| {
        text[..range.start]
            .chars()
            .next_back()
            .is_some_and(|ch| ch.is_ascii_alphanumeric() || ch == '_')
    })
}

/// A hole that is itself a relation or select-list identifier.
///
/// `` SELECT id FROM tmp_${expr} `` and `` SELECT ${name} `` qualify.
/// `` WHERE id = ${expr} `` does not: a placeholder there is a new column.
#[cfg(test)]
pub(crate) fn has_identifier_slot_hole(text: &str) -> bool {
    find_all(text)
        .iter()
        .any(|(range, _)| identifier_slot(text, range.start))
}

/// A hole whose replacement stays a value, as in `` WHERE id = ${expr} ``.
///
/// Restoring `sql_placeholder_N` keeps the statement parseable. Scrubbing the
/// hole leaves a trailing operator, and the identifier audit then treats the
/// configured helper as used.
#[cfg(test)]
pub(crate) fn has_value_operand_hole(text: &str) -> bool {
    find_all(text)
        .iter()
        .any(|(range, _)| value_operand(text, range.start))
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

#[cfg(test)]
fn value_operand(text: &str, hole_start: usize) -> bool {
    text[..hole_start].trim_end().ends_with('=')
}

/// Glued relation text, or a value operand, keeps a public placeholder
/// without a member site.
#[cfg(test)]
pub(crate) fn keeps_placeholder(text: &str) -> bool {
    has_value_operand_hole(text) || (splits_identifier(text) && has_identifier_slot_hole(text))
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

/// Put `sql_placeholder_N` back where each marker stood, in text order.
#[cfg(test)]
pub(crate) fn restore_placeholders(text: &str) -> String {
    let holes = find_all(text);
    if holes.is_empty() {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len());
    let mut cursor = 0;
    for (index, (range, _)) in holes.iter().enumerate() {
        out.push_str(&text[cursor..range.start]);
        out.push_str("sql_placeholder_");
        out.push_str(&(index + 1).to_string());
        cursor = range.end;
    }
    out.push_str(&text[cursor..]);
    out
}

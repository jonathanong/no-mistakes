//! sqlparser 0.63 rejects `GROUP BY DISTINCT ROLLUP/CUBE`.
//! A comma after DISTINCT keeps those grouping expressions parseable
//! without moving later lines.

use std::borrow::Cow;

mod opaque;
#[cfg(test)]
mod tests;

use opaque::{skip_comment, skip_opaque};

pub(super) fn separate_distinct_grouping(sql: &str) -> Cow<'_, str> {
    let edits = distinct_grouping_edits(sql);
    if edits.is_empty() {
        Cow::Borrowed(sql)
    } else {
        Cow::Owned(apply_edits(sql, &edits))
    }
}

struct Edit {
    at: usize,
    replace: usize,
}

fn distinct_grouping_edits(sql: &str) -> Vec<Edit> {
    let mut edits = Vec::new();
    let mut index = 0;
    while index < sql.len() {
        if let Some(next) = skip_opaque(sql, index) {
            index = next;
            continue;
        }
        if let Some((edit, next)) = distinct_grouping_at(sql, index) {
            edits.push(edit);
            index = next;
            continue;
        }
        index += sql[index..].chars().next().map_or(1, char::len_utf8);
    }
    edits
}

fn distinct_grouping_at(sql: &str, index: usize) -> Option<(Edit, usize)> {
    if !word_boundary_before(sql, index) {
        return None;
    }
    let mut at = match_word(sql, index, "group")?;
    at = skip_required_ws(sql, at)?;
    at = match_word(sql, at, "by")?;
    at = skip_required_ws(sql, at)?;
    at = match_word(sql, at, "distinct")?;
    let gap_start = at;
    let (at, last_break) = skip_grouping_gap(sql, at)?;
    let next = match_word(sql, at, "rollup").or_else(|| match_word(sql, at, "cube"))?;
    let edit = match last_break {
        Some(break_at) => Edit {
            at: break_at,
            replace: 1,
        },
        // Blank the first comment instead of inserting, so columns stay put.
        None => match first_comment(sql, gap_start, at) {
            Some((start, end)) => Edit {
                at: start,
                replace: end - start,
            },
            None => Edit {
                at: gap_start,
                replace: 0,
            },
        },
    };
    Some((edit, next))
}

fn apply_edits(sql: &str, edits: &[Edit]) -> String {
    let mut out = sql.to_string();
    for edit in edits.iter().rev() {
        let end = edit.at + edit.replace;
        // The comma takes the first byte; the rest of a blanked comment becomes
        // spaces, keeping its line breaks.
        let blanked: String = sql[edit.at..end]
            .chars()
            .skip(1)
            .map(|c| if matches!(c, '\n' | '\r') { c } else { ' ' })
            .collect();
        out.replace_range(edit.at..end, &format!(",{blanked}"));
    }
    out
}

fn word_boundary_before(sql: &str, index: usize) -> bool {
    index == 0
        || !sql[..index]
            .ends_with(|character: char| character.is_alphanumeric() || character == '_')
}

fn match_word(sql: &str, index: usize, word: &str) -> Option<usize> {
    let end = index.checked_add(word.len())?;
    let slice = sql.get(index..end)?;
    if !slice.eq_ignore_ascii_case(word) {
        return None;
    }
    if sql[end..].starts_with(|character: char| character.is_alphanumeric() || character == '_') {
        return None;
    }
    Some(end)
}

fn first_comment(sql: &str, start: usize, end: usize) -> Option<(usize, usize)> {
    (start..end).find_map(|index| skip_comment(sql, index).map(|next| (index, next)))
}

fn skip_required_ws(sql: &str, index: usize) -> Option<usize> {
    skip_grouping_gap(sql, index).map(|(next, _)| next)
}

fn skip_grouping_gap(sql: &str, mut index: usize) -> Option<(usize, Option<usize>)> {
    let start = index;
    let mut last_break = None;
    while let Some(character) = sql[index..].chars().next() {
        match character {
            ' ' | '\t' => {
                last_break = Some(index);
                index += 1;
            }
            '\n' | '\r' => index += 1,
            // Comments are whitespace to PostgreSQL; their newlines stay in place.
            '-' | '/' => match skip_comment(sql, index) {
                Some(next) => index = next,
                None => break,
            },
            _ => break,
        }
    }
    (index > start).then_some((index, last_break))
}

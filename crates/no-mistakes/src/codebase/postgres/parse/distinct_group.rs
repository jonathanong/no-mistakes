//! sqlparser 0.63 rejects `GROUP BY DISTINCT ROLLUP/CUBE`.
//! A comma after DISTINCT keeps those grouping expressions parseable
//! without moving later lines.

use std::borrow::Cow;

#[cfg(test)]
mod tests;

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
        None => Edit {
            at: gap_start,
            replace: 0,
        },
    };
    Some((edit, next))
}

fn apply_edits(sql: &str, edits: &[Edit]) -> String {
    let mut out = sql.to_string();
    for edit in edits.iter().rev() {
        let end = edit.at + edit.replace;
        out.replace_range(edit.at..end, ",");
    }
    out
}

fn word_boundary_before(sql: &str, index: usize) -> bool {
    index == 0
        || !sql[..index]
            .ends_with(|character: char| character.is_ascii_alphanumeric() || character == '_')
}

fn match_word(sql: &str, index: usize, word: &str) -> Option<usize> {
    let end = index.checked_add(word.len())?;
    let slice = sql.get(index..end)?;
    if !slice.eq_ignore_ascii_case(word) {
        return None;
    }
    if sql[end..]
        .starts_with(|character: char| character.is_ascii_alphanumeric() || character == '_')
    {
        return None;
    }
    Some(end)
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

fn skip_comment(sql: &str, index: usize) -> Option<usize> {
    match sql.as_bytes().get(index..index + 2)? {
        b"--" => Some(skip_line_comment(sql, index)),
        b"/*" => Some(skip_block_comment(sql, index)),
        _ => None,
    }
}

fn skip_opaque(sql: &str, index: usize) -> Option<usize> {
    let bytes = sql.as_bytes();
    match bytes.get(index)? {
        b'-' if bytes.get(index + 1) == Some(&b'-') => Some(skip_line_comment(sql, index)),
        b'/' if bytes.get(index + 1) == Some(&b'*') => Some(skip_block_comment(sql, index)),
        b'\'' | b'"' => Some(skip_quoted(sql, index, bytes[index])),
        b'$' => skip_dollar(sql, index),
        _ => None,
    }
}

fn skip_line_comment(sql: &str, index: usize) -> usize {
    sql[index..]
        .find('\n')
        .map(|offset| index + offset)
        .unwrap_or(sql.len())
}

fn skip_block_comment(sql: &str, index: usize) -> usize {
    sql[index + 2..]
        .find("*/")
        .map(|offset| index + 2 + offset + 2)
        .unwrap_or(sql.len())
}

fn skip_quoted(sql: &str, mut index: usize, quote: u8) -> usize {
    let bytes = sql.as_bytes();
    index += 1;
    while index < bytes.len() {
        if bytes[index] == quote {
            if bytes.get(index + 1) == Some(&quote) {
                index += 2;
                continue;
            }
            return index + 1;
        }
        index += 1;
    }
    index
}

fn skip_dollar(sql: &str, start: usize) -> Option<usize> {
    let bytes = sql.as_bytes();
    let mut index = start + 1;
    while index < bytes.len() && (bytes[index].is_ascii_alphanumeric() || bytes[index] == b'_') {
        index += 1;
    }
    if bytes.get(index) != Some(&b'$') {
        return None;
    }
    let tag = &sql[start..=index];
    let rest = index + 1;
    Some(
        sql[rest..]
            .find(tag)
            .map(|offset| rest + offset + tag.len())
            .unwrap_or(sql.len()),
    )
}

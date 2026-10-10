//! An opening label sits outside the occurrence: the walker skips it first.
//! Drop that label with the non-SQL span and leave the public span unchanged.
use super::super::body::Body;
use super::super::locations::Locations;
use super::super::types::PostgresSqlProceduralOccurrence;
use super::sql_occurrence;
use sqlparser::dialect::{Dialect, PostgreSqlDialect};
use sqlparser::tokenizer::{Token, TokenWithSpan, Whitespace};

pub(in super::super) fn opening_labels(
    body: &Body<'_>,
    occurrences: &[PostgresSqlProceduralOccurrence],
    tokens: &[TokenWithSpan],
    locations: &Locations<'_>,
) -> Vec<(usize, usize)> {
    let comments = tokens
        .iter()
        .filter(|token| {
            matches!(
                token.token,
                Token::Whitespace(
                    Whitespace::SingleLineComment { .. } | Whitespace::MultiLineComment(_)
                )
            )
        })
        .filter_map(|token| locations.span(token.span))
        .map(|span| (span.start.offset, span.end.offset))
        .collect::<Vec<_>>();
    let mut labels = Vec::new();
    for occurrence in occurrences {
        if sql_occurrence(occurrence.kind) {
            continue;
        }
        let Some(at) = decoded_index(body, occurrence.span.start.offset) else {
            continue;
        };
        let Some((start, end)) = label_before(body.sql.as_ref(), at, &comments) else {
            continue;
        };
        let start = body.offset(start).expect("label start");
        let end = body.offset(end).expect("label end");
        // A label already covered by another occurrence is not a second mark.
        if overlaps_any(start, end, occurrences) {
            continue;
        }
        labels.push((start, end));
    }
    labels
}

fn decoded_index(body: &Body<'_>, original: usize) -> Option<usize> {
    let mut lo = 0;
    let mut hi = body.sql.len();
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        if body.offset(mid).expect("decoded index") < original {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    (body.offset(lo).expect("decoded index") == original).then_some(lo)
}

fn overlaps_any(start: usize, end: usize, occurrences: &[PostgresSqlProceduralOccurrence]) -> bool {
    occurrences.iter().any(|occurrence| {
        (start < occurrence.span.end.offset && occurrence.span.start.offset < end)
            || overlaps_any(start, end, &occurrence.occurrences)
    })
}

/// `<<identifier>>` or the custom operator `<<>>`, with whitespace/comments before `at`.
fn label_before(sql: &str, at: usize, comments: &[(usize, usize)]) -> Option<(usize, usize)> {
    if !sql.is_char_boundary(at) {
        return None;
    }
    let end = skip_gap(sql, at, comments);
    if end >= 4 && sql[..end].ends_with("<<>>") && token_boundary(sql, end - 4, end, comments) {
        return Some((end - 4, end));
    }
    if end < 2 || !sql[..end].ends_with(">>") || !token_boundary(sql, end - 2, end, comments) {
        return None;
    }
    let ident_end = skip_gap(sql, end - 2, comments);
    let ident_start = identifier_start(sql, ident_end)?;
    let open = skip_gap(sql, ident_start, comments);
    if open < 2 || !sql[..open].ends_with("<<") || !token_boundary(sql, open - 2, open, comments) {
        return None;
    }
    Some((open - 2, end))
}

fn token_boundary(sql: &str, start: usize, end: usize, comments: &[(usize, usize)]) -> bool {
    !operator_before(sql, start, comments) && !operator_at(sql, end, comments)
}

fn operator_before(sql: &str, index: usize, comments: &[(usize, usize)]) -> bool {
    let at = comments.partition_point(|(_, end)| *end < index);
    if comments.get(at).is_some_and(|(_, end)| *end == index) {
        return false;
    }
    sql[..index].chars().next_back().is_some_and(is_operator)
}

fn operator_at(sql: &str, index: usize, comments: &[(usize, usize)]) -> bool {
    let at = comments.partition_point(|(start, _)| *start < index);
    if comments.get(at).is_some_and(|(start, _)| *start == index) {
        return false;
    }
    sql[index..].chars().next().is_some_and(is_operator)
}

fn is_operator(ch: char) -> bool {
    PostgreSqlDialect {}.is_custom_operator_part(ch)
}

fn skip_gap(sql: &str, mut at: usize, comments: &[(usize, usize)]) -> usize {
    loop {
        let before = at;
        while let Some((index, ch)) = prev_char(sql, at) {
            if !ch.is_ascii_whitespace() {
                break;
            }
            at = index;
        }
        let comment = comments.partition_point(|(_, end)| *end < at);
        if let Some((start, _)) = comments.get(comment).filter(|(_, end)| *end == at) {
            at = *start;
        }
        if at == before {
            return at;
        }
    }
}

fn identifier_start(sql: &str, end: usize) -> Option<usize> {
    if sql[..end].ends_with('"') {
        return quoted_identifier_start(sql, end);
    }
    let mut at = end;
    let mut start = None;
    while let Some((index, ch)) = prev_char(sql, at) {
        if is_identifier_part(ch) {
            start = Some(index);
            at = index;
            continue;
        }
        break;
    }
    let start = start?;
    is_identifier_start(sql[start..].chars().next()?).then_some(start)
}

fn quoted_identifier_start(sql: &str, end: usize) -> Option<usize> {
    let (close, _) = prev_char(sql, end)?;
    let mut at = close;
    loop {
        let (index, ch) = prev_char(sql, at)?;
        if ch == '"' {
            if let Some((prev, prev_ch)) = prev_char(sql, index) {
                if prev_ch == '"' {
                    at = prev;
                    continue;
                }
            }
            return Some(index);
        }
        at = index;
    }
}

fn prev_char(sql: &str, at: usize) -> Option<(usize, char)> {
    let ch = sql[..at].chars().next_back()?;
    Some((at - ch.len_utf8(), ch))
}

fn is_identifier_start(ch: char) -> bool {
    PostgreSqlDialect {}.is_identifier_start(ch)
}

fn is_identifier_part(ch: char) -> bool {
    PostgreSqlDialect {}.is_identifier_part(ch)
}

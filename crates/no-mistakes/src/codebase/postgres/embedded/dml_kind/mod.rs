//! Top-level DML kind after a complete `WITH` prefix, if any.
//!
//! Incomplete CTE lists and unknown leading keywords return `None` so callers
//! fail closed instead of claiming a reconstructed statement.

mod lex;

use lex::{skip_balanced_paren, skip_ident, skip_trivia, starts_keyword};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TopLevelDml {
    Select,
    Insert,
    Update,
    Delete,
    Merge,
}

impl TopLevelDml {
    pub(crate) fn is_insert_family(self) -> bool {
        matches!(self, Self::Insert | Self::Merge)
    }
}

/// True when recovered SQL is missing, incomplete, or a top-level INSERT/MERGE.
pub(crate) fn recovered_sql_needs_insert_check(sql: Option<&str>) -> bool {
    match sql {
        None => true,
        Some(sql) => top_level_dml_kind(sql).is_none_or(TopLevelDml::is_insert_family),
    }
}

pub(crate) fn top_level_dml_kind(sql: &str) -> Option<TopLevelDml> {
    let mut index = skip_trivia(sql, 0);
    if starts_keyword(sql, index, "with") {
        index = skip_complete_with(sql, index)?;
        index = skip_trivia(sql, index);
    }
    classify_dml(sql, index)
}

fn skip_complete_with(sql: &str, mut index: usize) -> Option<usize> {
    index += 4;
    index = skip_trivia(sql, index);
    if starts_keyword(sql, index, "recursive") {
        index += 9;
        index = skip_trivia(sql, index);
    }
    loop {
        index = skip_cte(sql, index)?;
        index = skip_trivia(sql, index);
        if sql.as_bytes().get(index) == Some(&b',') {
            index = skip_trivia(sql, index + 1);
            continue;
        }
        return Some(index);
    }
}

fn skip_cte(sql: &str, mut index: usize) -> Option<usize> {
    index = skip_ident(sql, index)?;
    index = skip_trivia(sql, index);
    if sql.as_bytes().get(index) == Some(&b'(') {
        index = skip_balanced_paren(sql, index)?;
        index = skip_trivia(sql, index);
    }
    if !starts_keyword(sql, index, "as") {
        return None;
    }
    index = skip_trivia(sql, index + 2);
    if starts_keyword(sql, index, "not") {
        index = skip_trivia(sql, index + 3);
        if !starts_keyword(sql, index, "materialized") {
            return None;
        }
        index = skip_trivia(sql, index + 12);
    } else if starts_keyword(sql, index, "materialized") {
        index = skip_trivia(sql, index + 12);
    }
    skip_balanced_paren(sql, index)
}

fn classify_dml(sql: &str, index: usize) -> Option<TopLevelDml> {
    const KINDS: &[(&str, TopLevelDml)] = &[
        ("select", TopLevelDml::Select),
        ("insert", TopLevelDml::Insert),
        ("update", TopLevelDml::Update),
        ("delete", TopLevelDml::Delete),
        ("merge", TopLevelDml::Merge),
    ];
    KINDS
        .iter()
        .find(|(keyword, _)| starts_keyword(sql, index, keyword))
        .map(|(_, kind)| *kind)
}

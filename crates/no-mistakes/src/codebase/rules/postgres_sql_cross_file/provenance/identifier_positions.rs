//! Placeholder indexes that occupy SQL identifier positions.
use crate::codebase::postgres::{parse_postgres_sql, EmbeddedSqlCall};
use sqlparser::ast::{ObjectName, Statement, Visit, Visitor};
use std::collections::BTreeSet;
use std::ops::ControlFlow;

mod walk;

pub(crate) fn scans_identifier(call: &EmbeddedSqlCall) -> bool {
    if super::super::sql::identifier_placeholder(call) == Some(true) {
        return true;
    }
    texts(call).into_iter().any(|sql| {
        parse_postgres_sql(sql).is_ok_and(|statements| !indexes(&statements).columns.is_empty())
    })
}

/// Identifier-position placeholder indexes and the highest placeholder index
/// in the recovered text. `None` when a variant does not parse.
pub(super) fn aligned(call: &EmbeddedSqlCall) -> Option<(BTreeSet<u32>, usize)> {
    let texts = texts(call);
    if texts.is_empty() {
        return None;
    }
    let mut max_index = 0usize;
    let mut identifiers = BTreeSet::new();
    for sql in texts {
        max_index = max_index.max(max_in_text(sql));
        let statements = parse_postgres_sql(sql).ok()?;
        let found = indexes(&statements);
        identifiers.extend(found.relations);
        identifiers.extend(found.columns);
    }
    if identifiers.is_empty() {
        return None;
    }
    Some((identifiers, max_index))
}

fn texts(call: &EmbeddedSqlCall) -> Vec<&str> {
    if call.variants.is_empty() {
        call.sql_text.iter().map(String::as_str).collect()
    } else {
        call.variants
            .iter()
            .map(|variant| variant.sql_text.as_str())
            .collect()
    }
}

fn max_in_text(sql: &str) -> usize {
    let marker = "sql_placeholder_";
    let mut rest = sql;
    let mut max_index = 0usize;
    while let Some(at) = rest.find(marker) {
        let after = &rest[at + marker.len()..];
        let digits = after.bytes().take_while(u8::is_ascii_digit).count();
        if let Ok(index) = after[..digits].parse::<usize>() {
            max_index = max_index.max(index);
        }
        rest = &after[digits..];
    }
    max_index
}

struct Indexes {
    relations: BTreeSet<u32>,
    columns: BTreeSet<u32>,
}

fn indexes(statements: &[Statement]) -> Indexes {
    let mut found = Indexes {
        relations: BTreeSet::new(),
        columns: BTreeSet::new(),
    };
    for statement in statements {
        let _ = statement.visit(&mut RelationIndexes {
            indexes: &mut found.relations,
        });
        walk::walk_statement(statement, &mut found);
    }
    found
}

struct RelationIndexes<'a> {
    indexes: &'a mut BTreeSet<u32>,
}

impl Visitor for RelationIndexes<'_> {
    type Break = ();
    fn pre_visit_relation(&mut self, relation: &ObjectName) -> ControlFlow<()> {
        walk::note_name(relation, self.indexes);
        ControlFlow::Continue(())
    }
}

#[cfg(test)]
mod tests;

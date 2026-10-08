use super::{parsing::ConflictFacts, project};
use crate::codebase::postgres::source::{locations::Locations, types::PostgresSqlStatementKind};
use sqlparser::{ast::Insert, parser::Parser};

pub(in crate::codebase::postgres::source) fn project_insert(
    value: &Insert,
    facts: Option<&ConflictFacts>,
    locations: &Locations<'_>,
    parser: &Parser,
    start_index: usize,
) -> PostgresSqlStatementKind {
    let tokens = (start_index..parser.index())
        .map(|index| parser.token_at(index).clone())
        .collect::<Vec<_>>();
    PostgresSqlStatementKind::Insert {
        insert: Box::new(project(value, facts, locations, &tokens)),
    }
}

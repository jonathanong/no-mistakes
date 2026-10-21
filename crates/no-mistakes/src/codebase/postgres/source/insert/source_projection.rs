use super::parsing::ConflictFacts;
use crate::codebase::postgres::source::{locations::Locations, types::PostgresSqlStatementKind};
use sqlparser::{ast::Insert, parser::Parser};

pub(in crate::codebase::postgres::source) fn project_insert(
    value: &Insert,
    facts: Option<&ConflictFacts>,
    cte_predicates: &[super::parsing::CtePredicate],
    locations: &Locations<'_>,
    parser: &Parser,
    start_index: usize,
) -> PostgresSqlStatementKind {
    let tokens = (start_index..parser.index())
        .map(|index| parser.token_at(index).clone())
        .collect::<Vec<_>>();
    PostgresSqlStatementKind::Insert {
        insert: Box::new(super::cte_projection::project_with_cte_predicates(
            value,
            facts,
            locations,
            &tokens,
            cte_predicates,
        )),
    }
}

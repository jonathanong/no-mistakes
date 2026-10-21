//! CTE inserts receive a copied conflict-target predicate.
//! The query collector owns the source query and the public RETURNING list.
use super::{parsing, project_inner};
use crate::codebase::postgres::source::{locations::Locations, types::PostgresSqlInsert};
use sqlparser::ast::Insert;
use sqlparser::tokenizer::TokenWithSpan;

pub(super) fn project_with_cte_predicates(
    value: &Insert,
    facts: Option<&parsing::ConflictFacts>,
    locations: &Locations<'_>,
    tokens: &[TokenWithSpan],
    cte_predicates: &[parsing::CtePredicate],
) -> PostgresSqlInsert {
    project_inner(value, facts, locations, false, tokens, cte_predicates)
}

pub(in crate::codebase::postgres::source) fn project_cte_core(
    value: &Insert,
    facts: Option<&parsing::ConflictFacts>,
    locations: &Locations<'_>,
) -> PostgresSqlInsert {
    project_inner(value, facts, locations, true, &[], &[])
}

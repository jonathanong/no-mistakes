use super::parse::{parse_postgres_sql, PostgresParseError};
use sqlparser::ast::{LimitClause, Query, Visit, Visitor};
use std::ops::ControlFlow;

/// Parse `sql` and report whether any query uses an `OFFSET` clause.
pub fn sql_has_offset_clause(sql: &str) -> Result<bool, PostgresParseError> {
    let statements = parse_postgres_sql(sql)?;
    let mut visitor = OffsetVisitor;
    Ok(statements
        .iter()
        .any(|statement| statement.visit(&mut visitor).is_break()))
}

struct OffsetVisitor;

impl Visitor for OffsetVisitor {
    type Break = ();

    fn pre_visit_query(&mut self, query: &Query) -> ControlFlow<Self::Break> {
        if limit_clause_has_offset(query.limit_clause.as_ref()) {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }
}

fn limit_clause_has_offset(clause: Option<&LimitClause>) -> bool {
    matches!(
        clause,
        Some(LimitClause::LimitOffset {
            offset: Some(_),
            ..
        }) | Some(LimitClause::OffsetCommaLimit { .. })
    )
}

#[cfg(test)]
mod tests;

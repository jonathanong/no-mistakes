mod locate;
#[cfg(test)]
mod tests;

use super::parse::{PostgresParseError, PreparedSql};
use super::statements::walk_executed;
use sqlparser::ast::{Expr, LimitClause, Query, Spanned, Statement, Value, Visit, Visitor};
use sqlparser::keywords::Keyword;
use sqlparser::tokenizer::Token;
use std::ops::ControlFlow;

/// One `OFFSET` clause. `Zero` is the integer literal `0`, including `OFFSET 0 ROWS`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OffsetUse {
    Zero,
    Other,
}

/// One executed OFFSET clause, at its keyword's source line and column.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SqlOffsetFact {
    pub line: usize,
    pub column: usize,
    pub kind: OffsetUse,
}

/// Parse `sql` and return every executed `OFFSET` clause in source order.
pub fn sql_offset_uses(sql: &str) -> Result<Vec<OffsetUse>, PostgresParseError> {
    let prepared = PreparedSql::new(sql);
    let statements = prepared.parse()?;
    Ok(offset_facts_prepared(prepared.normalized(), &statements)
        .into_iter()
        .map(|fact| fact.kind)
        .collect())
}

/// Parse `sql` and report whether any query uses an `OFFSET` clause.
pub fn sql_has_offset_clause(sql: &str) -> Result<bool, PostgresParseError> {
    Ok(!sql_offset_uses(sql)?.is_empty())
}

/// Executed OFFSET clauses in a SQL file, including DML and query wrappers.
pub fn sql_file_offset_uses(sql: &str) -> Vec<(usize, OffsetUse)> {
    super::statements::extract_sql_statement_facts(sql)
        .offset_uses
        .into_iter()
        .map(|fact| (fact.line, fact.kind))
        .collect()
}

pub(crate) fn offset_facts_prepared(
    normalized: &str,
    statements: &[Statement],
) -> Vec<SqlOffsetFact> {
    let mut collector = OffsetCollector::default();
    for statement in statements {
        let mut executed = Vec::new();
        walk_executed(statement, &mut executed);
        for statement in executed {
            let _ = statement.visit(&mut collector);
        }
    }
    if collector.uses.is_empty() {
        return Vec::new();
    }
    let tokens = super::parse::unicode::tokenize_raw_unicode(normalized);
    let keywords: Vec<_> = tokens
        .iter()
        .filter_map(|token| {
            matches!(&token.token, Token::Word(word) if word.keyword == Keyword::OFFSET).then_some(
                (
                    token.span.start.line as usize,
                    token.span.start.column as usize,
                ),
            )
        })
        .collect();
    for (index, fact) in collector.uses.iter_mut().enumerate() {
        if let Some((line, column)) =
            locate::resolve(normalized, &keywords, index, fact.line, fact.column)
        {
            fact.line = line;
            fact.column = column;
        }
        fact.line = fact.line.max(1);
    }
    collector.uses.sort_by_key(|fact| (fact.line, fact.column));
    collector.uses
}

#[derive(Default)]
struct OffsetCollector {
    uses: Vec<SqlOffsetFact>,
}

impl Visitor for OffsetCollector {
    type Break = ();
    fn pre_visit_query(&mut self, query: &Query) -> ControlFlow<Self::Break> {
        let value = match &query.limit_clause {
            Some(LimitClause::LimitOffset {
                offset: Some(offset),
                ..
            }) => Some(&offset.value),
            Some(LimitClause::OffsetCommaLimit { offset, .. }) => Some(offset),
            _ => None,
        };
        if let Some(value) = value {
            let position = value.span().start;
            self.uses.push(SqlOffsetFact {
                line: position.line as usize,
                column: position.column as usize,
                kind: if is_zero(value) {
                    OffsetUse::Zero
                } else {
                    OffsetUse::Other
                },
            });
        }
        ControlFlow::Continue(())
    }
}

fn is_zero(expr: &Expr) -> bool {
    match expr {
        Expr::Nested(inner) => is_zero(inner),
        Expr::Value(value) => matches!(&value.value, Value::Number(text, _) if text == "0"),
        _ => false,
    }
}

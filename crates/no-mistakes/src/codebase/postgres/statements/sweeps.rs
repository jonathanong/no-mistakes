//! Limit and key-walk facts for the bounded-iteration SQL shapes.
mod conjuncts;
mod page;
#[cfg(test)]
mod tests;

use super::limit::{is_limited, limit_site, Tokens};
use super::{walk_executed, SqlLimitFact, SqlLimitValue, SqlSweepFact};
use crate::codebase::postgres::idents::ident_key;
use sqlparser::ast::{DataType, Query, Statement, Visit, Visitor};
use std::collections::HashMap;
use std::ops::ControlFlow;

/// Every `LIMIT` / `FETCH FIRST` in the executed statements, and each limited single-table
/// query ordered by plain columns.
pub(super) fn collect(
    sql: &str,
    statements: &[Statement],
    recovered_placeholder_positions: &[(u32, u32)],
) -> (Vec<SqlLimitFact>, Vec<SqlSweepFact>) {
    let mut collector = Collector::new(sql, recovered_placeholder_positions);
    for statement in statements {
        let mut executed = Vec::new();
        walk_executed(statement, &mut executed);
        collector.transparent_int4_casts = transparent_int4_casts_safe(statement);
        for statement in executed {
            let _ = statement.visit(&mut collector);
        }
    }
    collector
        .limits
        .sort_by_key(|fact| (fact.line, fact.column));
    collector.sweeps.sort_by_key(|fact| fact.line);
    (collector.limits, collector.sweeps)
}

/// A declared non-int4 parameter can change value when cast to int4. Keep casts opaque for
/// that prepared statement rather than equating `$1::int` with `$1` in an expanded cursor.
fn transparent_int4_casts_safe(statement: &Statement) -> bool {
    match statement {
        Statement::Prepare {
            data_types,
            statement,
            ..
        } => {
            (data_types.is_empty()
                || data_types.iter().all(|data_type| {
                    matches!(
                        data_type,
                        DataType::Int(_) | DataType::Int4(_) | DataType::Integer(_)
                    )
                }))
                && transparent_int4_casts_safe(statement)
        }
        Statement::Explain { statement, .. } => transparent_int4_casts_safe(statement),
        _ => true,
    }
}

struct Collector<'a> {
    limits: Vec<SqlLimitFact>,
    sweeps: Vec<SqlSweepFact>,
    tokens: Tokens<'a>,
    /// The CTE names visible inside each open query: those of its enclosing queries, plus its own
    /// WITH clause for everything but the CTE bodies.
    scopes: Vec<Vec<String>>,
    /// The names a CTE body sees, set when its WITH clause is entered and keyed by the body.
    bodies: HashMap<*const Query, Vec<String>>,
    transparent_int4_casts: bool,
    recovered_placeholder_positions: &'a [(u32, u32)],
}

impl<'a> Collector<'a> {
    fn new(sql: &'a str, recovered_placeholder_positions: &'a [(u32, u32)]) -> Self {
        Self {
            limits: Vec::new(),
            sweeps: Vec::new(),
            tokens: Tokens::new(sql),
            scopes: Vec::new(),
            bodies: HashMap::new(),
            transparent_int4_casts: true,
            recovered_placeholder_positions,
        }
    }

    /// Open `query`'s scope. A non-recursive CTE sees only the CTEs declared before it; a
    /// recursive WITH shows every sibling and the CTE itself.
    fn enter(&mut self, query: &Query) {
        let outer = self
            .bodies
            .remove(&(query as *const Query))
            .or_else(|| self.scopes.last().cloned())
            .unwrap_or_default();
        let mut visible = outer.clone();
        if let Some(with) = &query.with {
            let names: Vec<String> = with
                .cte_tables
                .iter()
                .map(|cte| ident_key(&cte.alias.name))
                .collect();
            for (index, cte) in with.cte_tables.iter().enumerate() {
                let earlier = if with.recursive {
                    &names[..]
                } else {
                    &names[..index]
                };
                let mut body = outer.clone();
                body.extend(earlier.iter().cloned());
                self.bodies.insert(&*cte.query as *const Query, body);
            }
            visible.extend(names);
        }
        self.scopes.push(visible);
    }
}

impl Visitor for Collector<'_> {
    type Break = ();

    fn pre_visit_query(&mut self, query: &Query) -> ControlFlow<()> {
        self.enter(query);
        let site = limit_site(query, &self.tokens);
        // `LIMIT 0` returns nothing: it is a cap, but no page of a walk.
        let empty = site
            .as_ref()
            .is_some_and(|site| site.value == SqlLimitValue::Literal(0));
        if let Some(site) = site {
            self.limits.push(SqlLimitFact {
                line: site.line,
                column: site.column,
                value: site.value,
            });
        }
        if is_limited(query) && !empty {
            let visible = self.scopes.last().map_or(&[][..], Vec::as_slice);
            if let Some(sweep) = page::sweep(
                query,
                visible,
                self.transparent_int4_casts,
                self.recovered_placeholder_positions,
            ) {
                self.sweeps.push(sweep);
            }
        }
        ControlFlow::Continue(())
    }

    fn post_visit_query(&mut self, _: &Query) -> ControlFlow<()> {
        self.scopes.pop();
        ControlFlow::Continue(())
    }
}

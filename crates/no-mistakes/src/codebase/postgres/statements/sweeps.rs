//! Limit and key-walk facts for the bounded-iteration SQL shapes.
mod conjuncts;
mod page;
#[cfg(test)]
mod tests;

use super::limit::{is_empty_page, is_limited, limit_site, Tokens};
use super::{walk_executed, SqlLimitFact, SqlSweepFact};
use crate::codebase::postgres::idents::ident_key;
use sqlparser::ast::{Query, Statement, Visit, Visitor};
use std::collections::HashMap;
use std::ops::ControlFlow;

/// Every `LIMIT` / `FETCH FIRST` in the executed statements, and each limited single-table
/// query ordered by plain columns.
pub(super) fn collect(
    sql: &str,
    statements: &[Statement],
) -> (Vec<SqlLimitFact>, Vec<SqlSweepFact>) {
    let mut collector = Collector::new(sql);
    for statement in statements {
        let mut executed = Vec::new();
        walk_executed(statement, &mut executed);
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

struct Collector<'a> {
    limits: Vec<SqlLimitFact>,
    sweeps: Vec<SqlSweepFact>,
    tokens: Tokens<'a>,
    /// The CTE names visible inside each open query: those of its enclosing queries, plus its own
    /// WITH clause for everything but the CTE bodies.
    scopes: Vec<Vec<String>>,
    /// The names a CTE body sees, set when its WITH clause is entered and keyed by the body.
    bodies: HashMap<*const Query, Vec<String>>,
}

impl<'a> Collector<'a> {
    fn new(sql: &'a str) -> Self {
        Self {
            limits: Vec::new(),
            sweeps: Vec::new(),
            tokens: Tokens::new(sql),
            scopes: Vec::new(),
            bodies: HashMap::new(),
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
        // Transparent casts/unary signs can preserve zero without being bare literals.
        let empty = is_empty_page(query);
        if let Some(site) = site {
            self.limits.push(SqlLimitFact {
                line: site.line,
                column: site.column,
                value: site.value,
            });
        }
        if is_limited(query) && !empty {
            let visible = self.scopes.last().map_or(&[][..], Vec::as_slice);
            if let Some(sweep) = page::sweep(query, visible) {
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

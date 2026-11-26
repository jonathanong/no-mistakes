use super::{parse::PreparedSql, SqlFunctionCallFact, SqlStatementFileFacts};
use crate::fx::FxHashMap;
use sqlparser::ast::{Query, Spanned, Statement, Visit, Visitor};
use sqlparser::tokenizer::{Location, Span, TokenWithSpan};
use std::ops::ControlFlow;
use std::sync::Arc;

pub(super) mod tokens;
use tokens::Segments;

#[derive(Debug, Clone, PartialEq, Eq)]
#[doc(hidden)]
pub struct InventoryLockFact {
    pub category: String,
    pub fingerprint: String,
    pub line: usize,
    pub column: usize,
}

pub(super) fn unanalyzed_routines(statements: &[Statement]) -> Vec<usize> {
    statements
        .iter()
        .filter_map(|statement| {
            let name = match statement {
                Statement::CreateFunction(function) => &function.name,
                Statement::CreateProcedure { name, .. } => name,
                _ => return None,
            };
            Some(name.span().start.line.max(1) as usize)
        })
        .collect()
}

pub(super) struct Collector<'sql> {
    primary: Segments<'sql>,
    recovery: FxHashMap<(*const TokenWithSpan, usize), Segments<'sql>>,
    inventory: Vec<InventoryLockFact>,
}

pub(super) fn recovery_tokens<'a>(
    sources: Option<&'a [Option<Arc<[TokenWithSpan]>>]>,
    policy: Option<&'a [Option<Arc<[TokenWithSpan]>>]>,
) -> impl Fn(usize) -> Option<&'a [TokenWithSpan]> {
    move |index| {
        sources
            .and_then(|sources| sources.get(index)?.as_deref())
            .or_else(|| policy.and_then(|sources| sources.get(index)?.as_deref()))
    }
}

impl<'sql> Collector<'sql> {
    pub(super) fn new(prepared: &'sql PreparedSql<'_>) -> Self {
        Self {
            primary: Segments::new(prepared.tokens()),
            recovery: FxHashMap::default(),
            inventory: Vec::new(),
        }
    }

    pub(super) fn segments(&self) -> &Segments<'sql> {
        &self.primary
    }

    pub(super) fn collect(
        &mut self,
        statement: &Statement,
        recovery: Option<&'sql [TokenWithSpan]>,
        source_at: impl Fn(Span) -> Option<&'sql str>,
    ) {
        let fallback = recovery.map(|tokens| {
            &*self
                .recovery
                .entry((tokens.as_ptr(), tokens.len()))
                .or_insert_with(|| Segments::recovered(tokens, &source_at))
        });
        let start = statement.span().start;
        if self.primary.has_candidates_at(start)
            || fallback.is_some_and(|source| source.has_candidates_at(start))
        {
            let _ = statement.visit(&mut Scan {
                primary: &self.primary,
                fallback,
                inventory: &mut self.inventory,
            });
        }
    }

    pub(super) fn finish_projection(
        collector: Option<Self>,
        facts: &mut SqlStatementFileFacts,
        statements: &[Statement],
    ) {
        if let Some(collector) = collector {
            facts.lock_inventory = collector.finish(&facts.function_calls);
            facts.lock_inventory_unanalyzed = unanalyzed_routines(statements);
        }
    }

    fn finish<'call>(
        mut self,
        calls: impl IntoIterator<Item = &'call SqlFunctionCallFact>,
    ) -> Vec<InventoryLockFact> {
        let mut pending: Vec<_> = calls
            .into_iter()
            .filter(|call| {
                call.name_parts
                    .last()
                    .is_some_and(|name| advisory_name(name))
            })
            .filter(|call| !self.primary.advisory(call, &mut self.inventory))
            .collect();
        if !pending.is_empty() {
            pending.sort_unstable_by_key(|call| (call.line, call.column));
            for source in self.recovery.values() {
                source.advisories(&pending, &mut self.inventory);
            }
        }
        self.inventory.sort_unstable_by(|left, right| {
            (left.line, left.column, &left.category, &left.fingerprint).cmp(&(
                right.line,
                right.column,
                &right.category,
                &right.fingerprint,
            ))
        });
        self.inventory.dedup_by(|left, right| {
            (left.line, left.column, &left.category) == (right.line, right.column, &right.category)
        });
        self.inventory
    }
}

struct Scan<'a, 'sql> {
    primary: &'a Segments<'sql>,
    fallback: Option<&'a Segments<'sql>>,
    inventory: &'a mut Vec<InventoryLockFact>,
}

impl Visitor for Scan<'_, '_> {
    type Break = ();

    fn pre_visit_statement(&mut self, statement: &Statement) -> ControlFlow<()> {
        if let Statement::Lock(lock) = statement {
            if self.primary.table_lock(lock, self.inventory).is_none() {
                if let Some(fallback) = self.fallback {
                    fallback.table_lock(lock, self.inventory);
                }
            }
        }
        ControlFlow::Continue(())
    }

    fn pre_visit_query(&mut self, query: &Query) -> ControlFlow<()> {
        if !query.locks.is_empty() && !self.primary.row_locks(query, self.inventory) {
            if let Some(fallback) = self.fallback {
                fallback.row_locks(query, self.inventory);
            }
        }
        ControlFlow::Continue(())
    }
}

fn advisory_name(name: &str) -> bool {
    name.starts_with("pg_advisory_") || name.starts_with("pg_try_advisory_")
}

fn record(category: &str, statement: &str, at: Location, out: &mut Vec<InventoryLockFact>) {
    out.push(InventoryLockFact {
        category: category.to_owned(),
        fingerprint: format!("{category}:{statement}"),
        line: at.line as usize,
        column: at.column as usize,
    });
}

#[cfg(test)]
mod tests;

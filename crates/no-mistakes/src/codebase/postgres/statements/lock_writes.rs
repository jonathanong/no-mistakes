use crate::codebase::postgres::idents::{object_name_identity, unwrap_expr};
mod advisory;
mod helpers;
mod row_locks;
mod scope;
use super::value::PlaceholderPositions;
pub(crate) use advisory::AdvisoryKey;
use helpers::{access, generated_name};
use sqlparser::ast::{BinaryOperator, Expr, Query, Statement, TableFactor, Visit, Visitor};
use std::{collections::BTreeSet, ops::ControlFlow};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct LockWriteFacts {
    pub locks: Vec<RowAccess>,
    pub writes: Vec<RowAccess>,
    pub table_locks: Vec<Option<String>>,
    pub transaction_boundary: bool,
    pub relation_identity_changed: bool,
    pub advisory_keys: Vec<Vec<AdvisoryKey>>,
    pub self_contained_locks: BTreeSet<usize>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RowAccess {
    pub relation: Option<String>,
    pub key: Option<(String, String)>,
    pub has_predicate: bool,
}

pub(super) fn collect<const ADVANCED: bool>(
    statements: &[Statement],
    positions: PlaceholderPositions<'_>,
) -> LockWriteFacts {
    let mut out = LockWriteFacts::default();
    for statement in statements {
        if matches!(statement, Statement::Prepare { .. }) {
            continue;
        }
        let mut executed = Vec::new();
        super::wrappers::walk_executed(statement, &mut executed);
        for statement in executed {
            let (first_lock, first_write) = if ADVANCED {
                (out.locks.len(), out.writes.len())
            } else {
                (0, 0)
            };
            let _ = statement.visit(&mut Collector::<ADVANCED> {
                out: &mut out,
                scopes: Vec::new(),
                cte_depth: 0,
                with_stack: Vec::new(),
                positions,
            });
            if ADVANCED {
                scope::mark_self_contained(&mut out, first_lock, first_write);
            }
        }
    }
    out
}
struct Collector<'a, 'p, const ADVANCED: bool> {
    out: &'a mut LockWriteFacts,
    scopes: Vec<BTreeSet<String>>,
    cte_depth: usize,
    with_stack: Vec<bool>,
    positions: PlaceholderPositions<'p>,
}
impl<const ADVANCED: bool> Visitor for Collector<'_, '_, ADVANCED> {
    type Break = ();
    fn pre_visit_query(&mut self, query: &Query) -> ControlFlow<()> {
        if ADVANCED {
            self.scopes.push(scope::cte_names(query));
        } else {
            self.with_stack.push(query.with.is_some());
            self.cte_depth += usize::from(query.with.is_some());
        }
        if !query.locks.is_empty() {
            if ADVANCED {
                let mut locks = row_locks::collect_advanced(query, self.positions);
                for lock in &mut locks {
                    scope::hide_cte(lock, &self.scopes);
                }
                self.out.locks.extend(locks);
            } else {
                self.out.locks.push(row_locks::collect_legacy(
                    query,
                    self.positions,
                    self.cte_depth == 0,
                ));
            }
        }
        ControlFlow::Continue(())
    }
    fn post_visit_query(&mut self, _: &Query) -> ControlFlow<()> {
        if ADVANCED {
            self.scopes.pop().expect("balanced SQL query");
        } else {
            self.cte_depth -= usize::from(self.with_stack.pop().expect("balanced SQL query"));
        }
        ControlFlow::Continue(())
    }
    fn pre_visit_statement(&mut self, statement: &Statement) -> ControlFlow<()> {
        match statement {
            Statement::CreateTable(table) if table.temporary => {
                self.out.relation_identity_changed = true
            }
            Statement::Commit { .. } | Statement::Rollback { .. } => {
                self.out.transaction_boundary = true
            }
            Statement::Update(update) => {
                let mut target = access(
                    &update.table.relation,
                    update.selection.as_ref(),
                    self.positions,
                );
                if !update.table.joins.is_empty() || update.from.is_some() {
                    target.key = None;
                    target.has_predicate = true;
                }
                self.out.writes.push(target);
            }
            Statement::Delete(delete) => {
                use sqlparser::ast::FromTable;
                let (FromTable::WithFromKeyword(from) | FromTable::WithoutKeyword(from)) =
                    &delete.from;
                if let [table] = from.as_slice() {
                    let mut target =
                        access(&table.relation, delete.selection.as_ref(), self.positions);
                    if !table.joins.is_empty() || delete.using.is_some() {
                        target.key = None;
                        target.has_predicate = true;
                    }
                    self.out.writes.push(target);
                }
            }
            Statement::Lock(lock) => self.out.table_locks.extend(lock.tables.iter().map(|table| {
                (!generated_name(&table.name, self.positions))
                    .then(|| object_name_identity(&table.name))
            })),
            _ => {}
        }
        ControlFlow::Continue(())
    }
    fn pre_visit_expr(&mut self, expr: &Expr) -> ControlFlow<()> {
        if let Some(keys) = advisory::keys(expr, ADVANCED) {
            self.out.advisory_keys.push(keys);
        }
        ControlFlow::Continue(())
    }
}

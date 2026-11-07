use crate::codebase::postgres::idents::object_name_identity;
mod advisory;
mod helpers;
mod scope;
use super::value::PlaceholderPositions;
pub(crate) use advisory::AdvisoryKey;
use helpers::{access, generated_name, qualifier};
use sqlparser::ast::{
    BinaryOperator, Expr, Query, SetExpr, Statement, TableFactor, Visit, Visitor,
};
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

pub(super) fn collect(
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
            let first_lock = out.locks.len();
            let first_write = out.writes.len();
            let _ = statement.visit(&mut Collector {
                out: &mut out,
                scopes: Vec::new(),
                positions,
            });
            scope::mark_self_contained(&mut out, first_lock, first_write);
        }
    }
    out
}
struct Collector<'a, 'p> {
    out: &'a mut LockWriteFacts,
    scopes: Vec<BTreeSet<String>>,
    positions: PlaceholderPositions<'p>,
}
impl Visitor for Collector<'_, '_> {
    type Break = ();
    fn pre_visit_query(&mut self, query: &Query) -> ControlFlow<()> {
        self.scopes.push(scope::cte_names(query));
        if !query.locks.is_empty() {
            let access = match query.body.as_ref() {
                SetExpr::Select(select)
                    if select.from.len() == 1 && select.from[0].joins.is_empty() =>
                {
                    let mut row = access(
                        &select.from[0].relation,
                        select.selection.as_ref(),
                        self.positions,
                    );
                    scope::hide_cte(&mut row, &self.scopes);
                    row
                }
                _ => RowAccess {
                    relation: None,
                    key: None,
                    has_predicate: false,
                },
            };
            let valid_of = query.locks.iter().all(|lock| {
                lock.of.as_ref().is_none_or(|of| {
                    if generated_name(of, self.positions) {
                        return false;
                    }
                    if let SetExpr::Select(select) = query.body.as_ref() {
                        if let Some(from) = select.from.first() {
                            return qualifier(&from.relation)
                                .is_some_and(|name| name == object_name_identity(of));
                        }
                    }
                    false
                })
            });
            self.out.locks.push(if valid_of {
                access
            } else {
                RowAccess {
                    relation: None,
                    key: None,
                    has_predicate: false,
                }
            });
        }
        ControlFlow::Continue(())
    }
    fn post_visit_query(&mut self, _: &Query) -> ControlFlow<()> {
        self.scopes.pop().expect("balanced SQL query");
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
        if let Some(keys) = advisory::keys(expr) {
            self.out.advisory_keys.push(keys);
        }
        ControlFlow::Continue(())
    }
}

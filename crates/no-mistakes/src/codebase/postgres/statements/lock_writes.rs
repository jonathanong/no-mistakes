use crate::codebase::postgres::idents::{object_name_identity, unwrap_expr};
mod helpers;
use super::value::PlaceholderPositions;
use helpers::{access, constant, generated_name, qualifier};
use sqlparser::ast::{
    BinaryOperator, Expr, FunctionArg, FunctionArgExpr, FunctionArguments, Query, SetExpr,
    Statement, TableFactor, Visit, Visitor,
};
use std::ops::ControlFlow;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct LockWriteFacts {
    pub locks: Vec<RowAccess>,
    pub writes: Vec<RowAccess>,
    pub table_locks: Vec<Option<String>>,
    pub transaction_boundary: bool,
    pub relation_identity_changed: bool,
    pub advisory_keys: Vec<Vec<String>>,
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
            let _ = statement.visit(&mut Collector {
                out: &mut out,
                cte_depth: 0,
                with_stack: Vec::new(),
                positions,
            });
        }
    }
    out
}
struct Collector<'a, 'p> {
    out: &'a mut LockWriteFacts,
    cte_depth: usize,
    with_stack: Vec<bool>,
    positions: PlaceholderPositions<'p>,
}
impl Visitor for Collector<'_, '_> {
    type Break = ();
    fn pre_visit_query(&mut self, query: &Query) -> ControlFlow<()> {
        self.with_stack.push(query.with.is_some());
        self.cte_depth += usize::from(query.with.is_some());
        if !query.locks.is_empty() {
            let access = match query.body.as_ref() {
                SetExpr::Select(select)
                    if self.cte_depth == 0
                        && select.from.len() == 1
                        && select.from[0].joins.is_empty() =>
                {
                    access(
                        &select.from[0].relation,
                        select.selection.as_ref(),
                        self.positions,
                    )
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
        self.cte_depth -= usize::from(self.with_stack.pop().expect("balanced SQL query"));
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
        if let Expr::Function(function) = expr {
            let name = object_name_identity(&function.name);
            let name = name.strip_prefix("pg_catalog.").unwrap_or(&name);
            if matches!(
                name,
                "pg_advisory_lock"
                    | "pg_advisory_xact_lock"
                    | "pg_try_advisory_lock"
                    | "pg_try_advisory_xact_lock"
                    | "pg_advisory_lock_shared"
                    | "pg_advisory_xact_lock_shared"
                    | "pg_try_advisory_lock_shared"
                    | "pg_try_advisory_xact_lock_shared"
            ) {
                if let FunctionArguments::List(args) = &function.args {
                    let keys: Option<Vec<_>> = args.args.iter().map(|arg| match arg {
                        FunctionArg::Unnamed(FunctionArgExpr::Expr(expr)) if constant(expr) || matches!(unwrap_expr(expr), Expr::Value(value) if matches!(value.value, sqlparser::ast::Value::Placeholder(_))) => Some(expr.to_string()),
                        _ => None,
                    }).collect();
                    if let Some(keys) = keys.filter(|keys| matches!(keys.len(), 1 | 2)) {
                        self.out.advisory_keys.push(keys);
                    }
                }
            }
        }
        ControlFlow::Continue(())
    }
}

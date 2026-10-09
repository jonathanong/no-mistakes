use super::SqlFunctionClause;
use crate::fx::FxHashMap;
use sqlparser::ast::{Expr, OrderByExpr, SelectItem, SelectItemQualifiedWildcardKind};

mod joins;
mod mutations;
mod queries;

/// Expression identities belong to the borrowed AST and live only during its fact visit.
#[derive(Default)]
pub(super) struct Roots {
    expressions: FxHashMap<usize, Option<SqlFunctionClause>>,
}

impl Roots {
    // A stored None clears inherited context; a missing root inherits it.
    pub(super) fn clause(&self, expression: &Expr) -> Option<Option<SqlFunctionClause>> {
        self.expressions.get(&identity(expression)).copied()
    }

    fn expr(&mut self, expression: &Expr, clause: SqlFunctionClause) {
        self.expressions.insert(identity(expression), Some(clause));
    }

    fn unscoped(&mut self, expression: &Expr) {
        self.expressions.insert(identity(expression), None);
    }

    fn items(&mut self, items: &[SelectItem], clause: SqlFunctionClause) {
        for item in items {
            match item {
                SelectItem::UnnamedExpr(expression)
                | SelectItem::ExprWithAlias {
                    expr: expression, ..
                }
                | SelectItem::ExprWithAliases {
                    expr: expression, ..
                }
                | SelectItem::QualifiedWildcard(
                    SelectItemQualifiedWildcardKind::Expr(expression),
                    _,
                ) => self.expr(expression, clause),
                _ => {}
            }
        }
    }

    fn order_exprs(&mut self, expressions: &[OrderByExpr]) {
        for ordering in expressions {
            self.expr(&ordering.expr, SqlFunctionClause::OrderBy);
        }
    }
}

fn identity(expression: &Expr) -> usize {
    std::ptr::from_ref(expression).addr()
}

use super::SqlFunctionClause;
use crate::fx::FxHashMap;
use sqlparser::ast::{Expr, OrderByExpr, SelectItem, SelectItemQualifiedWildcardKind};

mod joins;
mod mutations;
mod queries;

/// Expression identities belong to the borrowed AST and live only during its fact visit.
#[derive(Default)]
pub(super) struct Roots {
    expressions: FxHashMap<usize, SqlFunctionClause>,
}

impl Roots {
    pub(super) fn clause(&self, expression: &Expr) -> Option<SqlFunctionClause> {
        self.expressions.get(&identity(expression)).copied()
    }

    fn expr(&mut self, expression: &Expr, clause: SqlFunctionClause) {
        self.expressions.insert(identity(expression), clause);
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

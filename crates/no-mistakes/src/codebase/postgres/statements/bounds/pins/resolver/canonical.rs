use super::Resolver;
use crate::codebase::postgres::catalog::canonical_bound_expr;
use sqlparser::ast::{visit_expressions_mut, Expr};
use std::ops::ControlFlow;

impl Resolver {
    /// Remove only qualifiers that this query resolves to `item`.
    pub(in super::super) fn canonical_for_item(
        &self,
        expression: &Expr,
        item: usize,
    ) -> Option<Expr> {
        let mut expression = expression.clone();
        let mut saw_column = false;
        let walk: ControlFlow<()> = visit_expressions_mut(&mut expression, |expr| {
            match expr {
                Expr::Identifier(_) | Expr::CompoundIdentifier(_) => {
                    let Some((owner, _)) = self.column(expr) else {
                        return ControlFlow::Break(());
                    };
                    if owner != item {
                        return ControlFlow::Break(());
                    }
                    saw_column = true;
                    if let Expr::CompoundIdentifier(parts) = expr {
                        *expr = Expr::Identifier(parts.last().expect("qualified column").clone());
                    }
                }
                Expr::Subquery(_) | Expr::Exists { .. } => return ControlFlow::Break(()),
                _ => {}
            }
            ControlFlow::Continue(())
        });
        (walk.is_continue() && saw_column).then(|| canonical_bound_expr(&expression))
    }
}

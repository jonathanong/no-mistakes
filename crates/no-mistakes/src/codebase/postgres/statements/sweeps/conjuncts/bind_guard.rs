use super::placeholders::is_recovered_placeholder;
use sqlparser::ast::{AccessExpr, Expr, Value, Visit, Visitor};
use std::ops::ControlFlow;

/// A guard can enable a walk, but cannot select individual rows.
pub(super) fn is_bind_guard(expr: &Expr, recovered_placeholder_positions: &[(u32, u32)]) -> bool {
    struct Guard<'a> {
        bind: bool,
        row_dependent: bool,
        field_labels: Vec<*const Expr>,
        recovered_placeholder_positions: &'a [(u32, u32)],
    }
    impl Visitor for Guard<'_> {
        type Break = ();
        fn pre_visit_expr(&mut self, expr: &Expr) -> ControlFlow<()> {
            if let Expr::CompoundFieldAccess { access_chain, .. } = expr {
                self.field_labels
                    .extend(access_chain.iter().filter_map(|access| match access {
                        AccessExpr::Dot(field @ Expr::Identifier(_)) => Some(field as *const Expr),
                        _ => None,
                    }));
            }
            match expr {
                Expr::Value(value) if matches!(value.value, Value::Placeholder(_)) => {
                    self.bind = true
                }
                Expr::QualifiedWildcard(..) => self.row_dependent = true,
                Expr::Identifier(_) if self.field_labels.contains(&(expr as *const Expr)) => {}
                Expr::Identifier(ident)
                    if is_recovered_placeholder(ident, self.recovered_placeholder_positions) =>
                {
                    self.bind = true
                }
                Expr::Identifier(_)
                | Expr::CompoundIdentifier(_)
                | Expr::Function(_)
                | Expr::Subquery(_)
                | Expr::Exists { .. }
                | Expr::InSubquery { .. } => self.row_dependent = true,
                _ => {}
            }
            ControlFlow::Continue(())
        }
    }
    let mut guard = Guard {
        bind: false,
        row_dependent: false,
        field_labels: Vec::new(),
        recovered_placeholder_positions,
    };
    let _ = expr.visit(&mut guard);
    guard.bind && !guard.row_dependent
}

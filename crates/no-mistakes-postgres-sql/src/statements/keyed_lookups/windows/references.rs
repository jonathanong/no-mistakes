use super::super::conditions::visit_case_children;
use super::{needs_original_frame, written_window_identity};
use crate::fx::{FxHashMap, FxHashSet};
use crate::idents::{function_arg_expr, ident_key};
use crate::statements::keyed_lookups::function_clauses::{
    coalesce_arguments, split_coalesce_arguments,
};
use sqlparser::ast::{Expr, Query, Spanned, Visitor, WindowSpec, WindowType};
use sqlparser::tokenizer::Location;
use std::ops::ControlFlow;

#[derive(Default)]
pub(super) struct WindowReferences {
    depth: usize,
    blocked_depth: usize,
    blocked_roots: FxHashSet<*const Expr>,
    pub(super) inactive: FxHashSet<Location>,
    pub(super) named: FxHashMap<String, bool>,
    pub(super) inline: FxHashMap<WindowSpec, Option<bool>>,
}

impl Visitor for WindowReferences {
    type Break = ();

    fn pre_visit_query(&mut self, _: &Query) -> ControlFlow<()> {
        self.depth += 1;
        ControlFlow::Continue(())
    }

    fn post_visit_query(&mut self, _: &Query) -> ControlFlow<()> {
        self.depth -= 1;
        ControlFlow::Continue(())
    }

    fn pre_visit_expr(&mut self, expr: &Expr) -> ControlFlow<()> {
        if self.depth == 0 {
            self.inspect_expression(expr);
        }
        ControlFlow::Continue(())
    }

    fn post_visit_expr(&mut self, _: &Expr) -> ControlFlow<()> {
        if self.depth == 0 {
            self.blocked_depth = self.blocked_depth.saturating_sub(1);
        }
        ControlFlow::Continue(())
    }
}

impl WindowReferences {
    fn inspect_expression(&mut self, expr: &Expr) {
        if self.blocked_depth > 0 || self.blocked_roots.remove(&std::ptr::from_ref(expr)) {
            self.blocked_depth += 1;
            if matches!(expr, Expr::Function(function) if function.over.is_some()) {
                let start = expr.span().start;
                if start.line > 0 {
                    self.inactive.insert(start);
                }
            }
            return;
        }

        match expr {
            Expr::Case {
                operand,
                conditions,
                else_result,
                ..
            } => {
                visit_case_children(
                    operand.as_deref(),
                    conditions,
                    else_result.as_deref(),
                    |child, reachable| {
                        if !reachable {
                            self.blocked_roots.insert(std::ptr::from_ref(child));
                        }
                    },
                );
            }
            Expr::Function(function) => {
                if let Some(arguments) = coalesce_arguments(function) {
                    let (_, skipped) = split_coalesce_arguments(arguments);
                    self.blocked_roots.extend(
                        skipped
                            .iter()
                            .filter_map(function_arg_expr)
                            .map(std::ptr::from_ref),
                    );
                }
                match &function.over {
                    Some(WindowType::NamedWindow(name)) => {
                        *self.named.entry(ident_key(name)).or_default() |=
                            needs_original_frame(function);
                    }
                    Some(WindowType::WindowSpec(spec)) => {
                        if let Some(name) = &spec.window_name {
                            self.named.entry(ident_key(name)).or_default();
                        }
                        if spec.window_frame.is_some() {
                            let needs_frame = self
                                .inline
                                .entry(written_window_identity(spec))
                                .or_default();
                            *needs_frame = Some(
                                needs_frame.unwrap_or_default() || needs_original_frame(function),
                            );
                        }
                    }
                    None => {}
                }
            }
            _ => {}
        }
    }
}

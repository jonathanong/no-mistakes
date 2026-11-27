use super::select_item_expr;
use crate::fx::FxHashSet;
use crate::idents::ident_key;
use sqlparser::ast::{Expr, NamedWindowExpr, Query, Select, Visit, Visitor, WindowType};
use std::ops::ControlFlow;

pub(super) fn referenced_expressions<'a>(
    select: &'a Select,
    order_by: &'a [&'a Expr],
) -> impl Iterator<Item = &'a Expr> {
    let mut references = WindowReferences::default();
    if !select.named_window.is_empty() {
        for expr in select
            .projection
            .iter()
            .filter_map(select_item_expr)
            .chain(order_by.iter().copied())
        {
            let _ = expr.visit(&mut references);
        }
        for definition in select.named_window.iter().rev() {
            if references.names.contains(&ident_key(&definition.0)) {
                let parent = match &definition.1 {
                    NamedWindowExpr::NamedWindow(name) => Some(name),
                    NamedWindowExpr::WindowSpec(spec) => spec.window_name.as_ref(),
                };
                references.names.extend(parent.map(ident_key));
            }
        }
    }
    select
        .named_window
        .iter()
        .filter_map(move |definition| match &definition.1 {
            NamedWindowExpr::WindowSpec(spec)
                if references.names.contains(&ident_key(&definition.0)) =>
            {
                Some(spec)
            }
            _ => None,
        })
        .flat_map(|spec| {
            spec.partition_by
                .iter()
                .chain(spec.order_by.iter().map(|item| &item.expr))
        })
}

#[derive(Default)]
struct WindowReferences {
    depth: usize,
    names: FxHashSet<String>,
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
            if let Expr::Function(function) = expr {
                let name = match &function.over {
                    Some(WindowType::NamedWindow(name)) => Some(name),
                    Some(WindowType::WindowSpec(spec)) => spec.window_name.as_ref(),
                    None => None,
                };
                self.names.extend(name.map(ident_key));
            }
        }
        ControlFlow::Continue(())
    }
}

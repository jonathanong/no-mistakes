use super::select_exprs::identity::written_window_identity;
use super::select_item_expr;
use crate::fx::{FxHashMap, FxHashSet};
use crate::idents::ident_key;
use sqlparser::ast::{
    Distinct, Expr, Function, FunctionArguments, NamedWindowExpr, Select, Spanned, Visit,
    WindowFrameBound, WindowSpec,
};
use sqlparser::tokenizer::Location;
use std::cell::RefCell;
mod references;
use references::WindowReferences;

pub(super) struct WindowPlan {
    named: FxHashMap<String, bool>,
    inactive: FxHashSet<Location>,
    inline: RefCell<FxHashMap<WindowSpec, InlineFrame>>,
}

#[derive(Clone, Copy)]
struct InlineFrame {
    direct: Option<bool>,
    nested: Option<bool>,
}

impl InlineFrame {
    fn new(needs_frame: Option<bool>) -> Self {
        Self {
            direct: needs_frame,
            nested: needs_frame,
        }
    }
}

impl WindowPlan {
    pub(super) fn new(select: &Select, order_by: &[&Expr]) -> Self {
        let distinct = match &select.distinct {
            Some(Distinct::On(expressions)) => expressions.as_slice(),
            _ => &[],
        };
        let mut references = WindowReferences::default();
        for expr in select
            .projection
            .iter()
            .filter_map(select_item_expr)
            .chain(distinct)
            .chain(order_by.iter().copied())
        {
            let _ = expr.visit(&mut references);
        }
        for definition in &select.named_window {
            if let NamedWindowExpr::WindowSpec(spec) = &definition.1 {
                if spec.window_frame.is_some() {
                    if let Some(needs_frame) = references
                        .inline
                        .get_mut(&written_window_identity(spec))
                        .and_then(Option::take)
                    {
                        *references
                            .named
                            .entry(ident_key(&definition.0))
                            .or_default() |= needs_frame;
                    }
                }
            }
        }
        for definition in select.named_window.iter().rev() {
            if let Some(&needs_frame) = references.named.get(&ident_key(&definition.0)) {
                let inherited = match &definition.1 {
                    NamedWindowExpr::NamedWindow(name) => Some((name, needs_frame)),
                    NamedWindowExpr::WindowSpec(spec) => {
                        spec.window_name.as_ref().map(|name| (name, false))
                    }
                };
                if let Some((name, needs_frame)) = inherited {
                    *references.named.entry(ident_key(name)).or_default() |= needs_frame;
                }
            }
        }
        Self {
            named: references.named,
            inactive: references.inactive,
            inline: RefCell::new(
                references
                    .inline
                    .into_iter()
                    .map(|(key, needs_frame)| (key, InlineFrame::new(needs_frame)))
                    .collect(),
            ),
        }
    }

    pub(super) fn named_expressions<'a>(
        &'a self,
        select: &'a Select,
    ) -> impl Iterator<Item = (&'a Expr, bool)> {
        select
            .named_window
            .iter()
            .filter_map(|definition| {
                let NamedWindowExpr::WindowSpec(spec) = &definition.1 else {
                    return None;
                };
                let needs_frame = self.named.get(&ident_key(&definition.0))?;
                Some((spec, *needs_frame))
            })
            .flat_map(|(spec, needs_frame)| window_expressions(spec, needs_frame))
    }

    pub(super) fn skips_call(&self, expr: &Expr) -> bool {
        self.inactive.contains(&expr.span().start)
    }

    pub(super) fn take_inline_frame(&self, spec: &WindowSpec, nested: bool) -> Option<bool> {
        if spec.window_frame.is_none() {
            return Some(false);
        }
        let mut inline = self.inline.borrow_mut();
        let frame = inline
            .entry(written_window_identity(spec))
            .or_insert_with(|| InlineFrame::new(Some(true)));
        if nested {
            frame.nested.take()
        } else {
            frame.direct.take()
        }
    }
}

pub(super) fn window_expressions(
    spec: &WindowSpec,
    needs_frame: bool,
) -> impl Iterator<Item = (&Expr, bool)> {
    let frame_bounds = spec
        .window_frame
        .iter()
        .filter(move |_| needs_frame)
        .flat_map(|frame| {
            std::iter::once(&frame.start_bound)
                .chain(frame.end_bound.iter())
                .filter_map(|bound| match bound {
                    WindowFrameBound::Preceding(expr) | WindowFrameBound::Following(expr) => {
                        expr.as_deref()
                    }
                    WindowFrameBound::CurrentRow => None,
                })
        });
    spec.partition_by
        .iter()
        .chain(spec.order_by.iter().map(|item| &item.expr))
        .map(|expr| (expr, true))
        .chain(frame_bounds.map(|expr| (expr, false)))
}

fn needs_original_frame(function: &Function) -> bool {
    let [schema, name] = function.name.0.as_slice() else {
        return true;
    };
    !(schema
        .as_ident()
        .is_some_and(|schema| ident_key(schema) == "pg_catalog")
        && name.as_ident().is_some_and(|name| {
            matches!(
                ident_key(name).as_str(),
                "row_number" | "rank" | "dense_rank" | "percent_rank" | "cume_dist"
            )
        })
        && matches!(&function.args, FunctionArguments::List(args) if args.args.is_empty()))
}

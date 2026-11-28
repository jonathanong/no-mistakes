use super::select_exprs::identity::{normalize_sort_options, normalize_target};
use super::select_item_expr;
use crate::fx::FxHashMap;
use crate::idents::ident_key;
use sqlparser::ast::{
    Distinct, Expr, Function, FunctionArguments, NamedWindowExpr, Query, Select, Visit, Visitor,
    WindowFrameBound, WindowSpec, WindowType,
};
use std::cell::RefCell;
use std::ops::ControlFlow;

pub(super) struct WindowPlan {
    named: FxHashMap<String, bool>,
    inline: RefCell<FxHashMap<WindowSpec, Option<bool>>>,
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
                        .get_mut(&window_identity(spec))
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
            inline: RefCell::new(references.inline),
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

    pub(super) fn take_inline_frame(&self, spec: &WindowSpec) -> Option<bool> {
        if spec.window_frame.is_none() {
            return Some(false);
        }
        self.inline
            .borrow_mut()
            .entry(window_identity(spec))
            .or_insert(Some(true))
            .take()
    }
}

fn window_identity(spec: &WindowSpec) -> WindowSpec {
    let mut key = spec.clone();
    normalize_sort_options(&mut key.order_by);
    normalize_target(&mut key);
    key
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
        && name
            .as_ident()
            .is_some_and(|name| ident_key(name) == "row_number")
        && matches!(&function.args, FunctionArguments::List(args) if args.args.is_empty()))
}

#[derive(Default)]
struct WindowReferences {
    depth: usize,
    named: FxHashMap<String, bool>,
    inline: FxHashMap<WindowSpec, Option<bool>>,
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
                            let needs_frame = self.inline.entry(window_identity(spec)).or_default();
                            *needs_frame = Some(
                                needs_frame.unwrap_or_default() || needs_original_frame(function),
                            );
                        }
                    }
                    None => {}
                }
            }
        }
        ControlFlow::Continue(())
    }
}

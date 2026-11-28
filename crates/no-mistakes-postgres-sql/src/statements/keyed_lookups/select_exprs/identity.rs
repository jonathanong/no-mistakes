use super::super::expr_norm::function::is_conditional_keyword;
use crate::idents::{ident_key, unwrap_expr};
use crate::order::canonical_order_options;
use sqlparser::ast::{
    CastKind, DataType, Distinct, Expr, Ident, OrderByExpr, OrderByKind, OrderBySort, Query,
    SetExpr, TableFactor, TableInheritance, VisitMut, VisitorMut, WindowSpec,
};
use std::ops::ControlFlow;
mod local_columns;
use local_columns::normalize_local_columns;

pub(in crate::statements::keyed_lookups) fn target_identity(expr: &Expr) -> Expr {
    let mut key = expr.clone();
    normalize(&mut key, true);
    key
}

pub(in crate::statements::keyed_lookups) fn written_window_identity(
    spec: &WindowSpec,
) -> WindowSpec {
    let mut key = spec.clone();
    normalize(&mut key, false);
    key
}

fn normalize(node: &mut impl VisitMut, normalize_targets: bool) {
    struct Identifiers {
        normalize_targets: bool,
        quoted_keyword_function: Option<*const Ident>,
    }
    impl VisitorMut for Identifiers {
        type Break = ();

        fn pre_visit_query(&mut self, query: &mut Query) -> ControlFlow<()> {
            if !self.normalize_targets {
                return ControlFlow::Continue(());
            }
            if let SetExpr::Select(select) = query.body.as_mut() {
                if matches!(select.distinct, Some(Distinct::All)) {
                    select.distinct = None;
                }
                normalize_local_columns(select);
            }
            if let Some(order) = &mut query.order_by {
                if let OrderByKind::Expressions(items) = &mut order.kind {
                    normalize_sort_options(items);
                }
            }
            ControlFlow::Continue(())
        }

        fn pre_visit_expr(&mut self, expr: &mut Expr) -> ControlFlow<()> {
            if matches!(expr, Expr::Nested(_)) {
                *expr = unwrap_expr(expr).clone();
            }
            if let Expr::Function(function) = expr {
                if let [name] = function.name.0.as_slice() {
                    self.quoted_keyword_function = name
                        .as_ident()
                        .filter(|name| {
                            name.quote_style.is_some() && is_conditional_keyword(&name.value)
                        })
                        .map(std::ptr::from_ref);
                }
            }
            if let Expr::Cast {
                kind, data_type, ..
            } = expr
            {
                if *kind == CastKind::DoubleColon {
                    *kind = CastKind::Cast;
                }
                if matches!(data_type, DataType::Int(None))
                    || self.normalize_targets && matches!(data_type, DataType::Int4(None))
                {
                    *data_type = DataType::Integer(None);
                }
            }
            ControlFlow::Continue(())
        }

        fn pre_visit_table_factor(&mut self, factor: &mut TableFactor) -> ControlFlow<()> {
            match factor {
                TableFactor::Table {
                    alias, inheritance, ..
                } => {
                    if matches!(inheritance, Some(TableInheritance::IncludeDescendants)) {
                        *inheritance = None;
                    }
                    if let Some(alias) = alias {
                        alias.explicit = false;
                    }
                }
                TableFactor::Derived {
                    alias: Some(alias), ..
                }
                | TableFactor::Function {
                    alias: Some(alias), ..
                }
                | TableFactor::UNNEST {
                    alias: Some(alias), ..
                }
                | TableFactor::NestedJoin {
                    alias: Some(alias), ..
                }
                | TableFactor::JsonTable {
                    alias: Some(alias), ..
                } => alias.explicit = false,
                _ => {}
            }
            ControlFlow::Continue(())
        }

        fn pre_visit_ident(&mut self, ident: &mut Ident) -> ControlFlow<()> {
            ident.value = ident_key(ident);
            if self.quoted_keyword_function == Some(std::ptr::from_ref(ident)) {
                self.quoted_keyword_function = None;
            } else {
                ident.quote_style = None;
            }
            ControlFlow::Continue(())
        }
    }

    let _ = node.visit(&mut Identifiers {
        normalize_targets,
        quoted_keyword_function: None,
    });
}

fn normalize_sort_options(order: &mut [OrderByExpr]) {
    for item in order {
        if let Some((ascending, nulls_first)) = canonical_order_options(&item.options) {
            item.options.sort = Some(if ascending {
                OrderBySort::Asc
            } else {
                OrderBySort::Desc
            });
            item.options.nulls_first = Some(nulls_first);
        }
    }
}

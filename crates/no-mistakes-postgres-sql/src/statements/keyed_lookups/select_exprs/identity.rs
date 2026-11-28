use super::super::table::lookup_table;
use crate::idents::{ident_key, object_name_identity, relation_suffix_name, unwrap_expr};
use crate::order::canonical_order_options;
use sqlparser::ast::{
    CastKind, Expr, Ident, OrderByExpr, OrderByKind, OrderBySort, Query, Select, SelectItem,
    SetExpr, TableFactor, VisitMut, VisitorMut,
};
use std::ops::ControlFlow;

pub(in crate::statements::keyed_lookups) fn target_identity(expr: &Expr) -> Expr {
    let mut key = expr.clone();
    normalize_target(&mut key);
    key
}

pub(super) fn normalize_target(node: &mut impl VisitMut) {
    struct Identifiers;
    impl VisitorMut for Identifiers {
        type Break = ();

        fn pre_visit_query(&mut self, query: &mut Query) -> ControlFlow<()> {
            if let SetExpr::Select(select) = query.body.as_mut() {
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
            if let Expr::Cast {
                kind: kind @ CastKind::DoubleColon,
                ..
            } = expr
            {
                *kind = CastKind::Cast;
            }
            ControlFlow::Continue(())
        }

        fn pre_visit_table_factor(&mut self, factor: &mut TableFactor) -> ControlFlow<()> {
            match factor {
                TableFactor::Table {
                    alias: Some(alias), ..
                }
                | TableFactor::Derived {
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
            ident.quote_style = None;
            ControlFlow::Continue(())
        }
    }

    let _ = node.visit(&mut Identifiers);
}

pub(super) fn normalize_sort_options(order: &mut [OrderByExpr]) {
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

fn normalize_local_columns(select: &mut Select) {
    let [table] = select.from.as_slice() else {
        return;
    };
    if !table.joins.is_empty() {
        return;
    }
    let Some((name, alias)) = lookup_table(&table.relation) else {
        return;
    };
    let qualifier = alias.map_or_else(
        || relation_suffix_name(&object_name_identity(name)),
        |alias| ident_key(&alias.name),
    );

    struct LocalColumns<'a> {
        qualifier: &'a str,
        depth: usize,
    }

    impl VisitorMut for LocalColumns<'_> {
        type Break = ();

        fn pre_visit_query(&mut self, _: &mut Query) -> ControlFlow<()> {
            self.depth += 1;
            ControlFlow::Continue(())
        }

        fn post_visit_query(&mut self, _: &mut Query) -> ControlFlow<()> {
            self.depth -= 1;
            ControlFlow::Continue(())
        }

        fn pre_visit_expr(&mut self, expr: &mut Expr) -> ControlFlow<()> {
            if self.depth == 0 {
                if let Expr::CompoundIdentifier(parts) = expr {
                    if let [qualifier, column] = parts.as_slice() {
                        if ident_key(qualifier) == self.qualifier {
                            *expr = Expr::Identifier(column.clone());
                        }
                    }
                }
            }
            ControlFlow::Continue(())
        }
    }

    let mut visitor = LocalColumns {
        qualifier: &qualifier,
        depth: 0,
    };
    for expr in select
        .projection
        .iter_mut()
        .filter_map(|item| match item {
            SelectItem::UnnamedExpr(expr) | SelectItem::ExprWithAlias { expr, .. } => Some(expr),
            _ => None,
        })
        .chain(select.selection.iter_mut())
    {
        let _ = expr.visit(&mut visitor);
    }
}

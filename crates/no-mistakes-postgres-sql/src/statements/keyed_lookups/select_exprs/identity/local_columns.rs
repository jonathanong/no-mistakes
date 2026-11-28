use super::super::super::table::lookup_table;
use crate::idents::{ident_key, object_name_identity, relation_suffix_name};
use sqlparser::ast::{Expr, Query, Select, SelectItem, VisitMut, VisitorMut};
use std::ops::ControlFlow;

pub(super) fn normalize_local_columns(select: &mut Select) {
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

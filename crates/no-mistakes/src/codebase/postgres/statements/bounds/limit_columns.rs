use super::super::value::PlaceholderPositions;
use super::Scope;
use crate::codebase::postgres::idents::{ident_key, object_name_ident, object_name_key};
use sqlparser::ast::{Expr, FunctionArg, FunctionArgExpr, Query, SetExpr, TableFactor};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Default)]
pub(super) struct Relations {
    names: BTreeMap<String, BTreeSet<String>>,
    opaque: bool,
}

impl Relations {
    pub(super) fn add(&mut self, factor: &TableFactor, positions: PlaceholderPositions<'_>) {
        let (alias, name, columns) = match factor {
            TableFactor::UNNEST {
                alias, array_exprs, ..
            } => {
                let columns = alias.as_ref().map_or_else(BTreeSet::new, |alias| {
                    alias
                        .columns
                        .iter()
                        .zip(array_exprs)
                        .filter(|(_, expr)| caller_input(expr, positions))
                        .map(|(column, _)| ident_key(&column.name))
                        .collect()
                });
                (alias, None, columns)
            }
            TableFactor::Function {
                alias, name, args, ..
            } => {
                let builtin = name.0.len() == 1
                    && object_name_ident(name).is_some_and(|id| ident_key(id) == "unnest");
                let columns = alias.as_ref().map_or_else(BTreeSet::new, |alias| {
                    alias
                        .columns
                        .iter()
                        .zip(args)
                        .filter_map(|(column, arg)| {
                            let FunctionArg::Unnamed(FunctionArgExpr::Expr(expr)) = arg else {
                                return None;
                            };
                            (builtin && caller_input(expr, positions))
                                .then(|| ident_key(&column.name))
                        })
                        .collect()
                });
                (alias, Some(object_name_key(name)), columns)
            }
            TableFactor::Table { alias, name, .. } => (
                alias,
                object_name_ident(name).map(ident_key),
                BTreeSet::new(),
            ),
            TableFactor::Derived { alias, .. } => (alias, None, BTreeSet::new()),
            _ => {
                self.names.clear();
                self.opaque = true;
                return;
            }
        };
        if let Some(name) = alias.as_ref().map(|alias| ident_key(&alias.name)).or(name) {
            self.names.insert(name, columns);
        } else {
            self.opaque = true;
        }
    }

    fn contains(&self, expr: &Expr) -> bool {
        match expr {
            Expr::CompoundIdentifier(parts) if parts.len() == 2 => self
                .names
                .get(&ident_key(&parts[0]))
                .is_some_and(|columns| columns.contains(&ident_key(&parts[1]))),
            Expr::Identifier(id) if !self.opaque && self.names.len() == 1 => self
                .names
                .values()
                .next()
                .is_some_and(|columns| columns.contains(&ident_key(id))),
            _ => false,
        }
    }
}

pub(super) fn is_limited(
    query: &Query,
    scope: &Scope,
    positions: PlaceholderPositions<'_>,
) -> bool {
    let mut visible = scope.limit_columns.clone();
    if let SetExpr::Select(select) = &*query.body {
        for table in &select.from {
            shadow(&mut visible, &table.relation, positions);
            for join in &table.joins {
                shadow(&mut visible, &join.relation, positions);
            }
        }
    } else {
        visible = Relations::default();
    }
    super::super::limit::is_limited_with_columns(query, positions, &|expr| visible.contains(expr))
}

fn shadow(visible: &mut Relations, factor: &TableFactor, positions: PlaceholderPositions<'_>) {
    let mut local = Relations::default();
    local.add(factor, positions);
    visible.opaque = true;
    for name in local.names.into_keys() {
        visible.names.insert(name, BTreeSet::new());
    }
    if local.opaque {
        visible.names.clear();
    }
}

fn caller_input(expr: &Expr, positions: PlaceholderPositions<'_>) -> bool {
    matches!(
        super::functions::unnest_kind_at(std::slice::from_ref(expr), positions),
        super::super::SqlBoundItemKind::Other
    )
}

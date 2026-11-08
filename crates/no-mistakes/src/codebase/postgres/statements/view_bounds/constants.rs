use super::scalar;
use crate::codebase::postgres::idents::{ident_key, object_name_ident};
use sqlparser::ast::{Expr, Query, SelectItem, SetExpr, TableFactor, VisitMut, VisitorMut};
use std::collections::BTreeMap;
use std::ops::ControlFlow;

type Columns = BTreeMap<String, Expr>;
type Ctes = BTreeMap<String, Columns>;

pub(super) fn inline(query: &Query, functions: &[String]) -> Query {
    let mut query = query.clone();
    let _ = query.visit(&mut Inline {
        functions,
        scopes: Vec::new(),
        cte_scopes: BTreeMap::new(),
    });
    query
}

struct Inline<'a> {
    functions: &'a [String],
    scopes: Vec<(Ctes, BTreeMap<String, Columns>)>,
    cte_scopes: BTreeMap<*const Query, Ctes>,
}

impl VisitorMut for Inline<'_> {
    type Break = ();

    fn pre_visit_query(&mut self, query: &mut Query) -> ControlFlow<()> {
        let mut ctes = self
            .cte_scopes
            .remove(&std::ptr::from_ref(query))
            .unwrap_or_else(|| {
                self.scopes
                    .last()
                    .map(|scope| scope.0.clone())
                    .unwrap_or_default()
            });
        if let Some(with) = &query.with {
            if with.recursive {
                for cte in &with.cte_tables {
                    ctes.remove(&ident_key(&cte.alias.name));
                }
            }
            for cte in &with.cte_tables {
                self.cte_scopes
                    .insert(std::ptr::from_ref(&*cte.query), ctes.clone());
                if !with.recursive {
                    let name = ident_key(&cte.alias.name);
                    ctes.remove(&name);
                    if let Some(columns) = columns(&cte.query, self.functions, &cte.alias.columns) {
                        ctes.insert(name, columns);
                    }
                }
            }
        }
        let mut bindings = BTreeMap::new();
        let mut counts = BTreeMap::<String, usize>::new();
        if let SetExpr::Select(select) = &*query.body {
            for factor in select.from.iter().flat_map(|from| {
                std::iter::once(&from.relation).chain(from.joins.iter().map(|join| &join.relation))
            }) {
                if let TableFactor::Table {
                    name,
                    alias,
                    args: None,
                    ..
                } = factor
                {
                    let Some(name) = object_name_ident(name) else {
                        continue;
                    };
                    let name = ident_key(name);
                    let qualifier = alias
                        .as_ref()
                        .map_or_else(|| name.clone(), |alias| ident_key(&alias.name));
                    *counts.entry(qualifier.clone()).or_default() += 1;
                    if alias.as_ref().is_none_or(|alias| alias.columns.is_empty()) {
                        if let Some(columns) = ctes.get(&name) {
                            bindings.insert(qualifier, columns.clone());
                        }
                    }
                }
            }
        }
        bindings.retain(|name, _| counts.get(name) == Some(&1));
        self.scopes.push((ctes, bindings));
        ControlFlow::Continue(())
    }

    fn post_visit_query(&mut self, _: &mut Query) -> ControlFlow<()> {
        self.scopes.pop();
        ControlFlow::Continue(())
    }

    fn pre_visit_expr(&mut self, expr: &mut Expr) -> ControlFlow<()> {
        if let Expr::CompoundIdentifier(parts) = expr {
            if let [qualifier, column] = parts.as_slice() {
                if let Some(value) = self
                    .scopes
                    .last()
                    .and_then(|scope| scope.1.get(&ident_key(qualifier)))
                    .and_then(|columns| columns.get(&ident_key(column)))
                {
                    *expr = value.clone();
                }
            }
        }
        ControlFlow::Continue(())
    }
}

fn columns(
    query: &Query,
    functions: &[String],
    aliases: &[sqlparser::ast::TableAliasColumnDef],
) -> Option<Columns> {
    if query.with.is_some() {
        return None;
    }
    let values: Vec<_> = match &*query.body {
        SetExpr::Values(values) if values.rows.len() == 1 => values.rows[0]
            .iter()
            .enumerate()
            .map(|(index, expr)| (format!("column{}", index + 1), expr.clone()))
            .collect(),
        SetExpr::Select(select) if select.from.is_empty() => select
            .projection
            .iter()
            .map(|item| match item {
                SelectItem::ExprWithAlias { expr, alias } => Some((ident_key(alias), expr.clone())),
                SelectItem::UnnamedExpr(expr) => Some((String::new(), expr.clone())),
                _ => None,
            })
            .collect::<Option<_>>()?,
        _ => return None,
    };
    if values
        .iter()
        .any(|(_, expr)| !scalar::scalar(expr, functions))
    {
        return None;
    }
    let mut columns = Columns::new();
    for (index, (name, expr)) in values.into_iter().enumerate() {
        let name = aliases
            .get(index)
            .map_or(name, |alias| ident_key(&alias.name));
        if name.is_empty() || columns.insert(name, expr).is_some() {
            return None;
        }
    }
    Some(columns)
}

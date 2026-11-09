use super::{
    analyze::{finding, Finding},
    expressions::*,
};
use sqlparser::ast::*;
use std::collections::BTreeMap;
pub(super) type Columns = Vec<String>;
fn output(body: &SetExpr) -> Option<Columns> {
    match body {
        SetExpr::Select(select) => select
            .projection
            .iter()
            .map(|item| match item {
                SelectItem::UnnamedExpr(expr) => column(expr).map(|(_, name)| name),
                SelectItem::ExprWithAlias { alias, .. } => Some(ident(alias)),
                _ => None,
            })
            .collect(),
        SetExpr::SetOperation { left, .. } => output(left),
        SetExpr::Query(query) => output(&query.body),
        _ => None,
    }
}
pub(super) fn columns(query: &Query) -> Option<Columns> {
    if query.with.as_ref().is_some_and(|with| with.recursive) {
        return None;
    }
    match query.body.as_ref() {
        SetExpr::SetOperation {
            op: SetOperator::Union,
            set_quantifier: SetQuantifier::None | SetQuantifier::Distinct,
            ..
        } => Some(output(&query.body).unwrap_or_default()),
        SetExpr::Query(query) => columns(query),
        _ => None,
    }
}
pub(super) fn outer(
    select: &Select,
    ctes: &BTreeMap<String, Columns>,
    scope: &super::scope::CatalogScope<'_>,
) -> bool {
    if select.distinct != Some(Distinct::Distinct) {
        return false;
    }
    let [from] = select.from.as_slice() else {
        return false;
    };
    if !from.joins.is_empty() {
        return false;
    }
    let (columns, alias) = match &from.relation {
        TableFactor::Derived {
            subquery, alias, ..
        } if alias.as_ref().is_none_or(|a| a.columns.is_empty()) => {
            if !super::evidence::catalogued(subquery, scope) {
                return false;
            }
            let Some(columns) = columns(subquery) else {
                return false;
            };
            (columns, alias.as_ref().map(|a| ident(&a.name)))
        }
        TableFactor::Table {
            name,
            alias,
            args: None,
            ..
        } if alias.as_ref().is_none_or(|a| a.columns.is_empty()) => {
            if name.0.len() != 1 {
                return false;
            }
            let key = crate::codebase::postgres::idents::object_name_key(name);
            let Some(columns) = ctes.get(&key) else {
                return false;
            };
            (
                columns.clone(),
                Some(alias.as_ref().map(|a| ident(&a.name)).unwrap_or(key)),
            )
        }
        _ => return false,
    };
    if select.projection.iter().any(|item| match item {
        SelectItem::Wildcard(options) => *options == WildcardAdditionalOptions::default(),
        SelectItem::QualifiedWildcard(
            SelectItemQualifiedWildcardKind::ObjectName(name),
            options,
        ) => {
            *options == WildcardAdditionalOptions::default()
                && alias.as_ref().is_some_and(|alias| {
                    crate::codebase::postgres::idents::object_name_key(name) == *alias
                })
        }
        _ => false,
    }) {
        return true;
    }
    let Some(projection) = projection(select) else {
        return false;
    };
    let projected = projection
        .iter()
        .filter_map(|expr| column(expr))
        .collect::<Vec<_>>();
    !columns.is_empty()
        && columns.iter().all(|name| {
            projected
                .iter()
                .any(|(owner, col)| col == name && (owner.is_none() || owner == &alias))
        })
}

pub(super) fn arms(body: &SetExpr, scope: &super::scope::CatalogScope<'_>, out: &mut Vec<Finding>) {
    match body {
        SetExpr::SetOperation {
            op: SetOperator::Union,
            set_quantifier: SetQuantifier::None | SetQuantifier::Distinct,
            left,
            right,
        } => {
            arm(left, scope, out);
            arm(right, scope, out);
        }
        SetExpr::Query(query) if query.limit_clause.is_none() && query.fetch.is_none() => {
            scope.in_query(query, |scope| arms(&query.body, scope, out))
        }
        _ => {}
    }
}
fn arm(body: &SetExpr, scope: &super::scope::CatalogScope<'_>, out: &mut Vec<Finding>) {
    match body {
        SetExpr::Select(select)
            if select.distinct == Some(Distinct::Distinct)
                && super::evidence::catalogued(select, scope) =>
        {
            out.push(finding(
                select,
                "distinct-over-union",
                "UNION removes duplicate arm rows; remove the redundant arm DISTINCT",
            ))
        }
        SetExpr::Query(query) if query.limit_clause.is_none() && query.fetch.is_none() => {
            scope.in_query(query, |scope| arm(&query.body, scope, out))
        }
        _ => arms(body, scope, out),
    }
}

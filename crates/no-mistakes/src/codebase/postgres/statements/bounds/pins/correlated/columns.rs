use super::super::super::super::value::is_placeholder_ident_at;
use crate::codebase::postgres::idents::ident_key;
use sqlparser::ast::{
    Expr, GroupByExpr, ObjectName, ObjectNamePart, OrderByKind, Query, SelectItem, SetExpr,
};
use std::collections::BTreeSet;

/// Projection labels are known only when every expression has a syntactic output name.
/// Stars and unnamed complex expressions preserve PostgreSQL's unknown-column fallback.
pub(in super::super::super) fn projection_columns(query: &Query) -> Option<BTreeSet<String>> {
    set_columns(&query.body)
}

fn set_columns(set: &SetExpr) -> Option<BTreeSet<String>> {
    match set {
        SetExpr::Select(select) => select.projection.iter().map(label_name).collect(),
        SetExpr::Query(query) => projection_columns(query),
        SetExpr::SetOperation { left, .. } => set_columns(left),
        _ => None,
    }
}

/// A syntactic output label can name a grouping expression in the same SELECT arm.
pub(super) fn label_name(item: &SelectItem) -> Option<String> {
    match item {
        SelectItem::ExprWithAlias { alias, .. } => Some(ident_key(alias)),
        SelectItem::UnnamedExpr(Expr::Identifier(ident)) => Some(ident_key(ident)),
        SelectItem::UnnamedExpr(Expr::CompoundIdentifier(parts)) => parts.last().map(ident_key),
        _ => None,
    }
}

/// Scalar built-ins use their declared OUT name, or otherwise the alias/function name.
/// Functions with unspecified record layouts remain unknown.
pub(super) fn function_columns(
    name: &ObjectName,
    alias: &Option<sqlparser::ast::TableAlias>,
) -> Option<BTreeSet<String>> {
    let function = match name.0.as_slice() {
        [ObjectNamePart::Identifier(function)] => function,
        [ObjectNamePart::Identifier(schema), ObjectNamePart::Identifier(function)]
            if ident_key(schema) == "pg_catalog" =>
        {
            function
        }
        _ => return None,
    };
    let function = ident_key(function);
    if ![
        "generate_series",
        "generate_subscripts",
        "unnest",
        "json_array_elements",
        "json_array_elements_text",
        "jsonb_array_elements",
        "jsonb_array_elements_text",
        "json_object_keys",
        "jsonb_object_keys",
        "jsonb_path_query",
        "jsonb_path_query_tz",
        "string_to_table",
        "regexp_split_to_table",
        "regexp_matches",
    ]
    .contains(&function.as_str())
    {
        return None;
    }
    let declared = matches!(
        function.as_str(),
        "json_array_elements"
            | "json_array_elements_text"
            | "jsonb_array_elements"
            | "jsonb_array_elements_text"
    );
    Some(
        [if declared {
            "value".to_string()
        } else {
            alias
                .as_ref()
                .map(|alias| ident_key(&alias.name))
                .unwrap_or(function)
        }]
        .into_iter()
        .collect(),
    )
}

/// The names written alone as an `ORDER BY` or `GROUP BY` item: PostgreSQL reads each as an
/// output column's name before it reads it as a column of a relation.
pub(super) fn output_names(query: &Query, positions: Option<&[(u32, u32)]>) -> Vec<String> {
    let mut items: Vec<&Expr> = Vec::new();
    if let Some(order) = &query.order_by {
        if let OrderByKind::Expressions(expressions) = &order.kind {
            items.extend(expressions.iter().map(|expression| &expression.expr));
        }
    }
    if let SetExpr::Select(select) = &*query.body {
        if let GroupByExpr::Expressions(expressions, _) = &select.group_by {
            items.extend(expressions);
        }
    }
    items
        .into_iter()
        .filter_map(|item| match item {
            Expr::Identifier(ident) if !is_placeholder_ident_at(ident, positions) => {
                Some(ident_key(ident))
            }
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests;

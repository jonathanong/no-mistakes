use crate::codebase::postgres::idents::ident_key;
use sqlparser::ast::{Expr, ObjectName, ObjectNamePart, Query, SelectItem, SetExpr};
use std::collections::BTreeSet;

/// Projection labels are known only when every expression has a syntactic output name.
/// Stars and unnamed complex expressions preserve PostgreSQL's unknown-column fallback.
pub(in super::super::super) fn projection_columns(query: &Query) -> Option<BTreeSet<String>> {
    set_columns(&query.body)
}

fn set_columns(set: &SetExpr) -> Option<BTreeSet<String>> {
    match set {
        SetExpr::Select(select) => select
            .projection
            .iter()
            .map(|item| match item {
                SelectItem::ExprWithAlias { alias, .. } => Some(ident_key(alias)),
                SelectItem::UnnamedExpr(Expr::Identifier(ident)) => Some(ident_key(ident)),
                SelectItem::UnnamedExpr(Expr::CompoundIdentifier(parts)) => {
                    parts.last().map(ident_key)
                }
                _ => None,
            })
            .collect(),
        SetExpr::Query(query) => projection_columns(query),
        SetExpr::SetOperation { left, .. } => set_columns(left),
        _ => None,
    }
}

/// Scalar built-in table functions expose a single column named by the alias or function.
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
    Some(
        [alias
            .as_ref()
            .map(|alias| ident_key(&alias.name))
            .unwrap_or(function)]
        .into_iter()
        .collect(),
    )
}

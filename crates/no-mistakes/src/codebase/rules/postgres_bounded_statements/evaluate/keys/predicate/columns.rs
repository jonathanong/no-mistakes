//! Column-type gates for casts PostgreSQL stores on varchar predicates.
use super::super::implied::text_family_name;
use crate::codebase::postgres::idents::{ident_key, unwrap_expr};
use crate::codebase::postgres::statements::{SqlBoundItem, SqlBoundItemKind};
use crate::codebase::postgres::SchemaCatalog;
use sqlparser::ast::{DataType, Expr};
use std::collections::BTreeMap;

pub(super) fn column_types(
    item: &SqlBoundItem,
    catalog: &SchemaCatalog,
) -> BTreeMap<String, String> {
    let SqlBoundItemKind::Table(name) = &item.kind else {
        return BTreeMap::new();
    };
    let Some(table) = catalog.table(name) else {
        return BTreeMap::new();
    };
    table
        .columns
        .iter()
        .map(|column| (column.name.clone(), column.data_type.clone()))
        .collect()
}

/// A bare column, or `(col)::text` when the catalog type compares as text.
pub(super) fn stored_column(expr: &Expr, types: &BTreeMap<String, String>) -> Option<String> {
    if let Some(name) = bare_column(expr) {
        return Some(name);
    }
    let Expr::Cast {
        expr, data_type, ..
    } = unwrap_expr(expr)
    else {
        return None;
    };
    if !matches!(data_type, DataType::Text) {
        return None;
    }
    let name = bare_column(expr)?;
    types
        .get(&name)
        .is_some_and(|data_type| text_family_name(data_type))
        .then_some(name)
}

pub(super) fn cast_fits(
    data_type: &DataType,
    column: Option<&str>,
    types: &BTreeMap<String, String>,
) -> bool {
    if !text_target(data_type) {
        return true;
    }
    let Some(column) = column else {
        return true;
    };
    match types.get(column) {
        Some(data_type) => text_family_name(data_type),
        None => true,
    }
}

fn bare_column(expr: &Expr) -> Option<String> {
    match unwrap_expr(expr) {
        Expr::Identifier(ident) => Some(ident_key(ident)),
        _ => None,
    }
}

fn text_target(data_type: &DataType) -> bool {
    matches!(
        data_type,
        DataType::Text
            | DataType::Varchar(_)
            | DataType::CharacterVarying(_)
            | DataType::Nvarchar(_)
            | DataType::CharVarying(_)
    )
}

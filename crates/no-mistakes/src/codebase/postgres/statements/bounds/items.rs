mod builder;
use super::super::value::PlaceholderPositions;
use super::Scope;
use crate::codebase::postgres::idents::ident_key;
use crate::codebase::postgres::statements::{SqlBoundItem, SqlBoundItemKind};
pub(super) use builder::Builder;
use sqlparser::ast::{ObjectName, Select};

pub(super) fn from_select(
    select: &Select,
    scope: &Scope,
    positions: PlaceholderPositions<'_>,
) -> Vec<SqlBoundItem> {
    // Projection subqueries are not row-bound inputs; their TABLE spellings cannot belong
    // to the FROM items we traverse below.
    if !select.from.is_empty() {
        scope.advance_table_tokens_to_from(select.select_token.0.span.start);
    }
    let mut builder = Builder::new(scope, positions);
    builder.tables(&select.from);
    builder.finish(select.selection.as_ref())
}

pub(super) fn other(at: (usize, usize)) -> SqlBoundItem {
    unnamed(SqlBoundItemKind::Other, at)
}

pub(super) fn opaque(at: (usize, usize)) -> SqlBoundItem {
    unnamed(SqlBoundItemKind::Opaque, at)
}

fn unnamed(kind: SqlBoundItemKind, at: (usize, usize)) -> SqlBoundItem {
    SqlBoundItem::new(kind, None, at)
}

/// A relation name as SQL, so the catalog lookup decodes it the way PostgreSQL would: an unquoted
/// part folds to lower case, and a quoted one keeps its case and any dots (`"Accounts"`).
pub(super) fn sql_name(name: &ObjectName) -> String {
    name.0
        .iter()
        .filter_map(|part| part.as_ident())
        .map(|ident| {
            if ident.quote_style.is_some() {
                format!("\"{}\"", ident.value.replace('"', "\"\""))
            } else {
                ident.value.to_ascii_lowercase()
            }
        })
        .collect::<Vec<_>>()
        .join(".")
}

fn alias_key(alias: &Option<sqlparser::ast::TableAlias>) -> Option<String> {
    alias.as_ref().map(|alias| ident_key(&alias.name))
}

fn alias_columns(alias: &Option<sqlparser::ast::TableAlias>) -> Vec<String> {
    alias.as_ref().map_or_else(Vec::new, |alias| {
        alias
            .columns
            .iter()
            .map(|column| ident_key(&column.name))
            .collect()
    })
}

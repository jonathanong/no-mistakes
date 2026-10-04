//! Catalog evidence for finite array constructor pins.
use crate::codebase::postgres::statements::{SqlBoundItemKind, SqlBoundQuery, SqlPinSource};
use crate::codebase::postgres::{decoded_parts, SchemaCatalog};
use sqlparser::{ast::DataType, dialect::PostgreSqlDialect, parser::Parser};
use std::collections::HashMap;

pub(super) fn pin_sources(query: &SqlBoundQuery, catalog: &SchemaCatalog) -> Vec<Vec<bool>> {
    // Parse each requested catalog type once, before fixed-point propagation.
    let mut scalar_types = HashMap::new();
    query
        .items
        .iter()
        .map(|item| {
            item.pins
                .iter()
                .map(|pin| match &pin.source {
                    SqlPinSource::Array {
                        items: _,
                        scalar_columns,
                        indexed_columns,
                        cast_types,
                    } => {
                        cast_types.iter().all(|ty| {
                            *scalar_types
                                .entry((ty.clone(), false))
                                .or_insert_with(|| scalar_type(ty, catalog))
                        }) && scalar_columns
                            .iter()
                            .map(|column| (column, false))
                            .chain(indexed_columns.iter().map(|column| (column, true)))
                            .all(|((index, column), indexed)| {
                                let source = &query.items[*index];
                                let SqlBoundItemKind::Table(name) = &source.kind else {
                                    return false;
                                };
                                let Some(table) = catalog.relation(name) else {
                                    return false;
                                };
                                if let Some(column) =
                                    column_type(table, &source.column_aliases, column)
                                {
                                    *scalar_types
                                        .entry((column.data_type.clone(), indexed))
                                        .or_insert_with(|| {
                                            if indexed {
                                                indexed_type(&column.data_type, catalog)
                                            } else {
                                                scalar_type(&column.data_type, catalog)
                                            }
                                        })
                                } else {
                                    !indexed
                                        && ["ctid", "tableoid", "xmin", "xmax", "cmin", "cmax"]
                                            .contains(&column.as_str())
                                        && !source.column_aliases.contains(column)
                                }
                            })
                    }
                    _ => true,
                })
                .collect()
        })
        .collect()
}

/// Unknown/domain types can hide an array; builtin scalar types and catalog-declared enums prove a leaf.
fn scalar_type(raw: &str, catalog: &SchemaCatalog) -> bool {
    let Ok(mut parser) = Parser::new(&PostgreSqlDialect {}).try_with_sql(raw) else {
        return false;
    };
    match parser.parse_data_type() {
        Ok(DataType::Array(_)) | Err(_) => false,
        Ok(DataType::Custom(name, args)) => {
            args.is_empty()
                && (builtin_scalar(&name.to_string())
                    || catalog.enum_type(&name.to_string()).is_some())
                && parser.peek_token().token == sqlparser::tokenizer::Token::EOF
        }
        Ok(_) => parser.peek_token().token == sqlparser::tokenizer::Token::EOF,
    }
}

// PostgreSQL dimensions describe one scalar element type, not nested SQL array values.
fn indexed_type(raw: &str, catalog: &SchemaCatalog) -> bool {
    use sqlparser::ast::ArrayElemTypeDef;
    let Ok(mut parser) = Parser::new(&PostgreSqlDialect {}).try_with_sql(raw) else {
        return false;
    };
    let Ok(DataType::Array(mut element)) = parser.parse_data_type() else {
        return false;
    };
    if parser.peek_token().token != sqlparser::tokenizer::Token::EOF {
        return false;
    }
    loop {
        let element_type = match element {
            ArrayElemTypeDef::SquareBracket(t, _) | ArrayElemTypeDef::Qualified(t, _) => t,
            _ => return false,
        };
        match *element_type {
            DataType::Array(inner) => element = inner,
            scalar => return scalar_type(&scalar.to_string(), catalog),
        }
    }
}

fn column_type<'a>(
    table: &'a crate::codebase::postgres::CatalogTable,
    aliases: &[String],
    requested: &str,
) -> Option<&'a crate::codebase::postgres::CatalogColumn> {
    // Visible columns follow catalog ordinals; dropped-column gaps do not add alias slots.
    if !aliases.is_empty()
        && (table
            .columns
            .iter()
            .any(|column| column.ordinal_position == 0)
            || table
                .columns
                .windows(2)
                .any(|pair| pair[0].ordinal_position == pair[1].ordinal_position))
    {
        return None;
    }
    let mut matches = table
        .columns
        .iter()
        .enumerate()
        .filter_map(|(index, column)| {
            (aliases
                .get(index)
                .map_or(column.name.as_str(), String::as_str)
                == requested)
                .then_some(column)
        });
    let column = matches.next()?;
    matches.next().is_none().then_some(column)
}

/// Keep existing key names only when a positional alias still denotes that catalog column.
pub(super) fn key_unchanged(
    table: &crate::codebase::postgres::CatalogTable,
    aliases: &[String],
    column: &str,
) -> bool {
    aliases.is_empty()
        || column_type(table, aliases, column).is_some_and(|entry| entry.name == column)
}

mod builtins;
use builtins::BUILTIN_SCALARS;

fn builtin_scalar(name: &str) -> bool {
    let parts = decoded_parts(name);
    let bare = match parts.as_slice() {
        [name] => name,
        [schema, name] if schema == "pg_catalog" => name,
        _ => return false,
    };
    BUILTIN_SCALARS.contains(&bare.as_str())
}

#[cfg(test)]
mod tests;

//! Name columns returned from `INSERT ... VALUES (...) RETURNING`.
use super::query::caller_value;
use super::returning::{name_returning, returning_name};
use crate::codebase::postgres::statements::SqlBoundOutput;
use crate::codebase::postgres::statements::SqlBoundQuery;
use sqlparser::ast::{Query, SelectItem, SetExpr};

pub(super) fn name(
    query: &mut SqlBoundQuery,
    source: &Query,
    returning: &[SelectItem],
    positions: super::super::value::PlaceholderPositions<'_>,
) {
    if query.outputs.len() == returning.len() {
        name_returning(query, returning);
        return;
    }
    let SetExpr::Values(values) = source.body.as_ref() else {
        return;
    };
    let Some(width) = values.rows.first().map(|row| row.content.len()) else {
        return;
    };
    if width != returning.len() || values.rows.iter().any(|row| row.content.len() != width) {
        return;
    }
    if !query.outputs.is_empty() {
        return;
    }
    query.outputs = (0..width)
        .map(|index| SqlBoundOutput {
            name: returning_name(&returning[index]),
            caller_sized: values
                .rows
                .iter()
                .all(|row| caller_value(&row.content[index], positions)),
            scalar: false,
            source_column: None,
            array_aggregate_column: None,
            constant_null: false,
        })
        .collect();
}

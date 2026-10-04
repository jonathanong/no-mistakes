//! Finite constructor leaves preserve source dependencies and scalar-type requirements.
use super::{merge, Resolver, Sourced};
use crate::codebase::postgres::SqlPinSource;
use sqlparser::ast::Expr;

mod caller_value;
mod columns;
pub(super) use caller_value::{caller_array, scalar_subquery};
mod fixed_boolean;
mod indexed;
mod leaves;
pub(super) use leaves::constructor;
mod scalar;
pub(super) use indexed::{argument_base, indexed_base};

/// Literal/bind leaves are caller-sized; a column leaf requires catalog scalar evidence.
/// Nested constructors flatten, so their leaves need the same proof rather than an arity guess.
pub(super) fn finite_array(
    elements: &[Expr],
    pinned: usize,
    resolver: &Resolver,
    positions: super::super::super::value::PlaceholderPositions<'_>,
) -> Option<Sourced> {
    let mut scalar_columns = Vec::new();
    let mut indexed_columns = Vec::new();
    let mut cast_types = Vec::new();
    let sources = elements
        .iter()
        .map(|element| {
            columns::collect(
                element,
                resolver,
                Some(&mut scalar_columns),
                &mut indexed_columns,
                &mut cast_types,
                false,
                positions,
            )?;
            resolver.source(element, pinned)
        })
        .collect::<Option<Vec<_>>>()?;
    let mut sourced = merge(sources);
    let items = match sourced.source {
        SqlPinSource::Items(items) => items,
        SqlPinSource::Value if !cast_types.is_empty() => Vec::new(),
        _ => return Some(sourced),
    };
    {
        scalar_columns.sort();
        scalar_columns.dedup();
        indexed_columns.sort();
        indexed_columns.dedup();
        cast_types.sort();
        cast_types.dedup();
        sourced.source = SqlPinSource::Array {
            items,
            scalar_columns,
            indexed_columns,
            cast_types,
        };
    }
    Some(sourced)
}

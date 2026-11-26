//! Exact constraint spans use the enclosing prepared statement token ranges.
mod alter;
mod create;
mod tokens;
pub(super) use alter::{alter_ranges, alter_span};
pub(super) use create::{column_span, column_span_in_range, table_span};
pub(super) use tokens::table_ranges;

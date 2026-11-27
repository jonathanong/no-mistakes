pub(crate) use super::view_bounds::{bounded_view_selects, BOUNDED_VIEW_MARKER};
pub use no_mistakes_postgres_sql::statements::*;

/// Extract statement facts from SQL recovered by [`crate::codebase::postgres::extract_embedded_sql_from_source`].
///
/// This forwards the call's exact generated-placeholder positions to bound, value-form, and
/// relation-restriction facts, distinguishing interpolated binds from user-written identifiers
/// with the same marker spelling. Returns `None` when the call has no recovered SQL. Fact locations remain
/// relative to the recovered SQL text; for a `Dynamic` call, that text may contain only a
/// verified leading statement fragment.
pub fn extract_sql_statement_facts_for_embedded_call(
    call: &super::EmbeddedSqlCall,
) -> Option<SqlStatementFileFacts> {
    let sql = call.sql_text.as_deref()?;
    Some(extract_sql_statement_facts_with_recovered_placeholders(
        sql,
        true,
        &call.recovered_placeholder_positions,
    ))
}

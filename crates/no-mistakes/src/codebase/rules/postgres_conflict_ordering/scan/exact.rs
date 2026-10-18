use super::analysis;
use crate::codebase::postgres::{SchemaCatalog, SqlStatementFileFacts};
use crate::codebase::rules::RuleFinding;

/// Preserve the exact-call finding shape while reusing the request's one SQL
/// parse and conflict projection.
pub(super) fn findings(
    file: &str,
    line: usize,
    sql: &str,
    statement: &SqlStatementFileFacts,
    catalog: &SchemaCatalog,
    fail_unanalyzable: bool,
) -> Vec<RuleFinding> {
    let locations = statement.variant_locations.as_ref();
    if statement.parse_failed
        || locations.is_none_or(|locations| locations.conflict_error.is_some())
    {
        if fail_unanalyzable && analysis::contains_insert_conflict(sql) {
            return vec![analysis::finding(
                file,
                line,
                "unanalyzable-sql",
                "keep INSERT ... ON CONFLICT SQL statically parseable so canonical ordering can be checked",
            )];
        }
        return Vec::new();
    }
    analysis::findings_for_inserts(file, line, &locations.unwrap().conflicts, catalog)
        .into_iter()
        .map(|(_, finding)| finding)
        .collect()
}

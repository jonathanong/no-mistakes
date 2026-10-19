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
    let locations = statement
        .variant_locations
        .as_ref()
        .expect("conflict locations");
    let prepared = super::gate::prepared_conflicts(locations, statement.parse_failed);
    let mut findings = Vec::new();
    if prepared.unanalyzable && fail_unanalyzable && analysis::contains_insert_conflict(sql) {
        findings.push(analysis::finding(
            file,
            line,
            "unanalyzable-sql",
            "keep INSERT ... ON CONFLICT SQL statically parseable so canonical ordering can be checked",
        ));
    }
    findings.extend(
        analysis::findings_for_inserts(file, line, prepared.conflicts, catalog)
            .into_iter()
            .map(|(_, finding)| finding),
    );
    findings
}

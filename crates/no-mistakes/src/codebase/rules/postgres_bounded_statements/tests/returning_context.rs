use super::{catalog, fixture_root};
use crate::codebase::postgres::extract_sql_statement_facts;
use crate::codebase::rules::postgres_bounded_statements::evaluate::offenders;

#[test]
fn returning_queries_use_catalog_target_proofs_and_preserve_negative_controls() {
    let sql = std::fs::read_to_string(fixture_root().join("sql/returning-context.sql")).unwrap();
    let catalog = catalog();
    let found = extract_sql_statement_facts(&sql);
    assert!(!found.parse_failed, "{found:?}");
    let lines: std::collections::BTreeSet<_> = found
        .bounds
        .iter()
        .filter(|fact| !offenders(fact, &catalog).is_empty())
        .map(|fact| fact.line)
        .collect();
    assert_eq!(lines, [2, 4, 6, 7].into_iter().collect());
}

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

#[test]
fn keyed_target_reports_opaque_recordset_driver_root() {
    let sql = std::fs::read_to_string(fixture_root().join("sql/target-opaque-driver.sql")).unwrap();
    let found = extract_sql_statement_facts(&sql);
    assert!(!found.parse_failed, "{found:?}");
    let catalog = catalog();
    let targets: Vec<_> = found
        .bounds
        .iter()
        .filter(|fact| fact.target.is_some())
        .collect();
    assert_eq!(targets.len(), 1, "{found:?}");
    let roots: Vec<_> = offenders(targets[0], &catalog)
        .into_iter()
        .map(|offender| offender.table)
        .collect();
    assert_eq!(roots, ["accounts"]);
}

#[test]
fn temporary_dml_target_has_no_permanent_relation_offender() {
    use crate::codebase::postgres::statements::SqlBoundItemKind;
    let sql = std::fs::read_to_string(fixture_root().join("sql/target-temporary.sql")).unwrap();
    let found = extract_sql_statement_facts(&sql);
    assert!(!found.parse_failed, "{found:?}");
    assert_eq!(found.bounds.len(), 1, "{found:?}");
    let fact = &found.bounds[0];
    let target = fact.target.expect("DELETE target");
    assert!(matches!(
        fact.query.items[target].kind,
        SqlBoundItemKind::Opaque
    ));
    assert!(offenders(fact, &catalog()).is_empty());
}

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

#[test]
fn nested_composite_writes_and_named_drivers_preserve_key_ownership() {
    use crate::codebase::postgres::SchemaCatalog;
    let root = fixture_root().join("../issue-3114-exact/fixture");
    let sources = crate::codebase::rules::source_store_for_files(&[root.join("r5-schema.json")]);
    let catalog = SchemaCatalog::load(&root, "r5-schema.json", &sources).unwrap();
    let sql = std::fs::read_to_string(root.join("r5.sql")).unwrap();
    let facts = extract_sql_statement_facts(&sql);
    assert!(!facts.parse_failed);
    let lines: std::collections::BTreeSet<_> = facts
        .bounds
        .iter()
        .filter(|fact| !offenders(fact, &catalog).is_empty())
        .map(|fact| fact.line)
        .collect();
    assert_eq!(lines, [27, 30, 38, 60, 64, 68].into_iter().collect());
}

#[test]
fn returning_column_names_do_not_grant_value_or_pushdown_proofs() {
    use crate::codebase::postgres::SqlBoundItemKind;
    let root = fixture_root().join("../issue-3114-exact/fixture");
    let sql = std::fs::read_to_string(root.join("r5.sql")).unwrap();
    let facts = extract_sql_statement_facts(&sql);
    let pushed = facts.bounds.iter().find(|fact| fact.line == 6).unwrap();
    let SqlBoundItemKind::Query(done) = &pushed.query.items[1].kind else {
        panic!("returning driver");
    };
    assert_eq!(
        done.outputs
            .iter()
            .filter_map(|output| output.name.as_deref())
            .collect::<Vec<_>>(),
        ["tenant_id", "entry_id"]
    );
    assert!(done.outputs.iter().all(|output| !output.caller_sized
        && !output.scalar
        && output.source_column.is_none()
        && output.array_aggregate_column.is_none()
        && !output.recursive_domain
        && !output.constant_null));
}

#[test]
fn a_materialized_lock_driver_keeps_the_exact_nested_write_bounded() {
    let root = fixture_root().join("../issue-3114-exact/fixture");
    let sources = crate::codebase::rules::source_store_for_files(&[root.join("r5-schema.json")]);
    let catalog =
        crate::codebase::postgres::SchemaCatalog::load(&root, "r5-schema.json", &sources).unwrap();
    let sql = std::fs::read_to_string(root.join("r5-lock-chain.sql")).unwrap();
    let facts = extract_sql_statement_facts(&sql);
    assert!(!facts.parse_failed);
    assert!(facts
        .bounds
        .iter()
        .filter(|fact| fact.line < 15)
        .all(|fact| offenders(fact, &catalog).is_empty()));
    assert!(facts
        .bounds
        .iter()
        .filter(|fact| fact.line >= 15)
        .any(|fact| !offenders(fact, &catalog).is_empty()));
}

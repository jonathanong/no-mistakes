use super::offenders;
use crate::codebase::postgres::{extract_sql_statement_facts, SchemaCatalog};

#[test]
fn explicit_database_temp_source_survives_ambiguous_namesake_in_same_view() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-mixed-current-database-view.sql"
    ));
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/schema-current-database.json"
    ));
    let facts = extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    assert_eq!(facts.bounds.len(), 1);
    for (database, expected) in [("Audit.Database", vec!["accounts"]), ("other", vec![])] {
        let mut snapshot: serde_json::Value = serde_json::from_str(raw).unwrap();
        snapshot["currentDatabase"] = database.into();
        let catalog = SchemaCatalog::from_json(&snapshot.to_string()).unwrap();
        let names = facts
            .bounds
            .iter()
            .flat_map(|fact| offenders(fact, &catalog))
            .map(|offender| offender.table)
            .collect::<Vec<_>>();
        assert_eq!(names, expected, "database={database}");
    }
}

#[test]
fn temporary_database_identity_requires_explicit_catalog_match() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-current-database.sql"));
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/schema-current-database.json"
    ));
    let facts = extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    assert_eq!(facts.bounds.len(), 16);
    for (database, expected) in [
        (Some("Audit.Database"), vec![4, 8, 9, 11, 12, 13]),
        (Some("other"), vec![0, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13]),
        (None, vec![0, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13]),
    ] {
        let mut snapshot: serde_json::Value = serde_json::from_str(raw).unwrap();
        match database {
            Some(database) => {
                snapshot["currentDatabase"] = database.into();
            }
            None => {
                snapshot.as_object_mut().unwrap().remove("currentDatabase");
            }
        }
        let catalog = SchemaCatalog::from_json(&snapshot.to_string()).unwrap();
        let found = facts
            .bounds
            .iter()
            .enumerate()
            .filter_map(|(index, fact)| (!offenders(fact, &catalog).is_empty()).then_some(index))
            .collect::<Vec<_>>();
        assert_eq!(found, expected, "database={database:?}");
    }
}

#[test]
fn qualified_temporary_views_and_folded_database_names_use_same_prepared_facts() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-current-database-views.sql"));
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/schema-current-database.json"
    ));
    let facts = extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    assert_eq!(facts.bounds.len(), 9);
    for (database, expected) in [
        ("Audit.Database", vec![2, 4, 6, 7, 8]),
        ("auditdb", vec![0, 1, 2, 3, 4, 5, 6, 8]),
        ("AuditDB", (0..9).collect()),
    ] {
        let mut snapshot: serde_json::Value = serde_json::from_str(raw).unwrap();
        snapshot["currentDatabase"] = database.into();
        let catalog = SchemaCatalog::from_json(&snapshot.to_string()).unwrap();
        let found = facts
            .bounds
            .iter()
            .enumerate()
            .filter_map(|(index, fact)| (!offenders(fact, &catalog).is_empty()).then_some(index))
            .collect::<Vec<_>>();
        assert_eq!(found, expected, "database={database:?}");
    }
}

#[test]
fn database_and_search_path_evidence_must_both_match() {
    let facts = extract_sql_statement_facts(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-current-database-search-path.sql")));
    assert!(!facts.parse_failed);
    assert_eq!(facts.bounds.len(), 4);
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/schema-current-database.json"
    ));
    let mut snapshot: serde_json::Value = serde_json::from_str(raw).unwrap();
    snapshot["searchPathEvidence"] =
        serde_json::json!({"pg_catalog": [], "empty_schema": [], "public": ["accounts"]});
    let catalog = SchemaCatalog::from_json(&snapshot.to_string()).unwrap();
    let found = facts
        .bounds
        .iter()
        .enumerate()
        .filter_map(|(index, fact)| (!offenders(fact, &catalog).is_empty()).then_some(index))
        .collect::<Vec<_>>();
    assert_eq!(found, [1, 3]);
    snapshot["currentDatabase"] = "other".into();
    let catalog = SchemaCatalog::from_json(&snapshot.to_string()).unwrap();
    assert!(facts
        .bounds
        .iter()
        .all(|fact| !offenders(fact, &catalog).is_empty()));
}

#[test]
fn conditional_partition_lifetimes_reuse_facts_across_catalog_databases() {
    let facts = extract_sql_statement_facts(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-current-database-partitions.sql")));
    // GenericDialect rejects ATTACH/DETACH; the narrow statement recovery is intentional.
    assert!(facts.parse_failed);
    assert_eq!(facts.bounds.len(), 6);
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/schema-current-database.json"
    ));
    for (database, expected) in [
        ("Audit.Database", vec![2, 3, 5]),
        ("other", (0..6).collect()),
    ] {
        let mut snapshot: serde_json::Value = serde_json::from_str(raw).unwrap();
        snapshot["currentDatabase"] = database.into();
        let catalog = SchemaCatalog::from_json(&snapshot.to_string()).unwrap();
        let found = facts
            .bounds
            .iter()
            .enumerate()
            .filter_map(|(index, fact)| (!offenders(fact, &catalog).is_empty()).then_some(index))
            .collect::<Vec<_>>();
        assert_eq!(found, expected, "database={database:?}");
    }
}

#[test]
fn wrong_kind_materialized_drops_preserve_catalog_condition() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-current-database-materialized.sql"));
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/schema-current-database.json"
    ));
    let facts = extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    for (database, expected) in [("Audit.Database", vec![2, 4]), ("other", (0..5).collect())] {
        let mut snapshot: serde_json::Value = serde_json::from_str(raw).unwrap();
        snapshot["currentDatabase"] = database.into();
        let catalog = SchemaCatalog::from_json(&snapshot.to_string()).unwrap();
        let found = facts
            .bounds
            .iter()
            .enumerate()
            .filter_map(|(index, fact)| (!offenders(fact, &catalog).is_empty()).then_some(index))
            .collect::<Vec<_>>();
        assert_eq!(found, expected, "database={database}");
    }
}

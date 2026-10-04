use super::offenders;
use crate::codebase::postgres::{extract_sql_statement_facts, SchemaCatalog};

#[test]
fn proven_physical_shadow_drop_exposes_the_underlying_temporary_rows() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-shadow-drop.sql"
    ));
    let catalog = SchemaCatalog::from_json(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/schema-search-path-evidence.json"
    )))
    .unwrap();
    assert_eq!(
        crate::codebase::postgres::parse_postgres_sql(sql)
            .unwrap()
            .len(),
        13
    );
    let facts = extract_sql_statement_facts(sql);
    let names = crate::codebase::postgres::project_sql_bounds(&facts, &catalog)
        .iter()
        .flat_map(|fact| offenders(fact, &catalog))
        .map(|offender| offender.table)
        .collect::<Vec<_>>();
    assert_eq!(names, ["orders", "orders"]);
}

#[test]
fn unknown_shadow_drop_never_lends_catalog_keys_to_the_join() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-shadow-drop-unknown.sql"
    ));
    let catalog = SchemaCatalog::from_json(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/schema-search-path-evidence.json"
    )))
    .unwrap();
    assert_eq!(
        crate::codebase::postgres::parse_postgres_sql(sql)
            .unwrap()
            .len(),
        10
    );
    let facts = extract_sql_statement_facts(sql);
    let names = crate::codebase::postgres::project_sql_bounds(&facts, &catalog)
        .iter()
        .flat_map(|fact| offenders(fact, &catalog))
        .map(|offender| offender.table)
        .collect::<Vec<_>>();
    assert_eq!(names, ["accounts", "orders", "accounts", "orders"]);
}

#[test]
fn database_qualified_shadow_drop_requires_a_matching_current_database() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-shadow-drop-database.sql"
    ));
    assert_eq!(
        crate::codebase::postgres::parse_postgres_sql(sql)
            .unwrap()
            .len(),
        5
    );
    let facts = extract_sql_statement_facts(sql);
    for (snapshot, expected) in [
        (
            "schema-search-path-current-database.json",
            Vec::<&str>::new(),
        ),
        (
            "schema-search-path-evidence.json",
            vec!["accounts", "orders"],
        ),
    ] {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-bounded-statements/fixture")
            .join(snapshot);
        let catalog = SchemaCatalog::from_json(&std::fs::read_to_string(path).unwrap()).unwrap();
        let names = crate::codebase::postgres::project_sql_bounds(&facts, &catalog)
            .iter()
            .flat_map(|fact| offenders(fact, &catalog))
            .map(|offender| offender.table)
            .collect::<Vec<_>>();
        assert_eq!(names, expected, "{snapshot}");
    }
}

#[test]
fn recreated_physical_shadows_never_retire_the_underlying_temp_relation() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-bounded-statements/fixture");
    let catalog = SchemaCatalog::from_json(
        &std::fs::read_to_string(root.join("schema-shadow-recreate.json")).unwrap(),
    )
    .unwrap();
    for (fixture, expected) in [
        (
            "temporary-shadow-recreate-table.sql",
            vec!["accounts", "orders"],
        ),
        ("temporary-shadow-recreate-view.sql", vec!["orders"]),
        ("temporary-shadow-recreate-materialized.sql", vec!["orders"]),
    ] {
        let sql = std::fs::read_to_string(root.join("sql").join(fixture)).unwrap();
        assert_eq!(
            crate::codebase::postgres::parse_postgres_sql(&sql)
                .unwrap()
                .len(),
            7
        );
        let facts = extract_sql_statement_facts(&sql);
        let names = crate::codebase::postgres::project_sql_bounds(&facts, &catalog)
            .iter()
            .flat_map(|fact| offenders(fact, &catalog))
            .map(|offender| offender.table)
            .collect::<Vec<_>>();
        assert_eq!(names, expected, "{fixture}");
    }
}

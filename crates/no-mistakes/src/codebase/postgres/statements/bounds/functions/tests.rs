use super::{scalar::SCALAR, SET_RETURNING};

#[test]
fn every_catalog_set_returning_function_is_known() {
    let Ok(connection) = std::env::var("NO_MISTAKES_TEST_POSTGRES_URL") else {
        assert!(
            std::env::var_os("CI").is_none(),
            "CI must supply the real PostgreSQL lane"
        );
        return;
    };
    let mut command = std::process::Command::new("psql");
    crate::postgres_catalog::connection_environment(&connection, &mut command).unwrap();
    let output = command.args(["-X", "--no-password", "-At", "-v", "ON_ERROR_STOP=1", "-c",
                "SELECT DISTINCT proname FROM pg_proc WHERE pronamespace = 'pg_catalog'::regnamespace AND proretset ORDER BY proname"])
            .output().unwrap();
    assert!(output.status.success(), "pg_proc query failed");
    let names = String::from_utf8(output.stdout).unwrap();
    let missing: Vec<_> = names
        .lines()
        .filter(|name| !SET_RETURNING.contains(name))
        .collect();
    assert!(
        missing.is_empty(),
        "unclassified set-returning built-ins: {missing:?}"
    );
}

#[test]
fn trusted_scalar_builtin_inventory_has_scalar_projection_facts() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/scalar-projection-builtins.sql"));
    let facts = crate::codebase::postgres::extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    assert_eq!(facts.bounds.len(), SCALAR.len());
    assert!(facts.bounds.iter().all(|fact| fact
        .query
        .items
        .iter()
        .all(|item| matches!(item.kind, super::SqlBoundItemKind::Other))));
}

#[test]
fn trusted_scalar_builtin_inventory_matches_postgres_cardinality() {
    let Ok(connection) = std::env::var("NO_MISTAKES_TEST_POSTGRES_URL") else {
        assert!(
            std::env::var_os("CI").is_none(),
            "CI must supply the real PostgreSQL lane"
        );
        return;
    };
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/catalog/scalar-builtin-catalog.sql"));
    let mut command = std::process::Command::new("psql");
    crate::postgres_catalog::connection_environment(&connection, &mut command).unwrap();
    let output = command
        .args([
            "-X",
            "--no-password",
            "-At",
            "-v",
            "ON_ERROR_STOP=1",
            "-c",
            sql,
        ])
        .output()
        .unwrap();
    assert!(output.status.success(), "function catalog query failed");
    let names = String::from_utf8(output.stdout).unwrap();
    let catalog: std::collections::BTreeMap<_, _> = names
        .lines()
        .map(|line| line.split_once('|').unwrap())
        .collect();
    for name in SCALAR {
        assert_eq!(
            catalog.get(name),
            Some(&"f"),
            "trusted scalar builtin {name} must exist and have no set-returning overload"
        );
    }
}

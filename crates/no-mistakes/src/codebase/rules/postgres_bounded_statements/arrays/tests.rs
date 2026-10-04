use super::{scalar_type, BUILTIN_SCALARS};

#[test]
fn builtin_scalar_type_proofs_preserve_catalog_namespace_and_array_boundaries() {
    let root = super::super::tests::fixture_root();
    let sources = crate::codebase::rules::source_store_for_files(&[root.join("schema.json")]);
    let catalog =
        crate::codebase::postgres::SchemaCatalog::load(&root, "schema.json", &sources).unwrap();
    let cases: Vec<(String, bool)> = serde_json::from_str(
        &std::fs::read_to_string(root.join("finite-builtin-scalar-types.json")).unwrap(),
    )
    .unwrap();
    for (ty, expected) in cases {
        assert_eq!(scalar_type(&ty, &catalog), expected, "{ty}");
    }
}

#[test]
fn builtin_scalar_type_inventory_matches_postgres_array_cardinality() {
    let Ok(connection) = std::env::var("NO_MISTAKES_TEST_POSTGRES_URL") else {
        assert!(
            std::env::var_os("CI").is_none(),
            "CI must supply the real PostgreSQL lane"
        );
        return;
    };
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/catalog/finite-builtin-scalar-types.sql"));
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
    assert!(
        output.status.success(),
        "PostgreSQL type inventory query failed"
    );
    let names = String::from_utf8(output.stdout).unwrap();
    assert_eq!(names.lines().collect::<Vec<_>>(), BUILTIN_SCALARS);
}

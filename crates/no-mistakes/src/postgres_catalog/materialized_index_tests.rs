use super::*;

#[test]
fn generated_catalog_retains_materialized_view_indexes() {
    let Some(connection) = std::env::var_os("NO_MISTAKES_TEST_POSTGRES_URL") else {
        assert!(
            std::env::var_os("CI").is_none(),
            "CI must supply PostgreSQL"
        );
        return;
    };
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/postgres/catalog-materialized-view/fixture/schema.sql");
    let mut command = Command::new("psql");
    connection_environment(&connection.to_string_lossy(), &mut command).unwrap();
    let output = command
        .args(["-X", "--no-password", "-v", "ON_ERROR_STOP=1", "-f"])
        .arg(fixture)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "saved catalog fixture must execute"
    );
    let options = PostgresCatalogOptions {
        connection_env: "NO_MISTAKES_TEST_POSTGRES_URL".into(),
        schema: "Catalog.Materialized".into(),
        coverage: PostgresCatalogCoverage::Complete,
        search_path_schemas: Vec::new(),
        current_database: None,
    };
    let complete = generate(&options).unwrap();
    let view = &complete["views"]["recent_events"];
    assert_eq!(view["materialized"], true);
    let indexes = view["indexes"].as_object().unwrap();
    assert_eq!(indexes.len(), 2);
    let refresh = &indexes["recent_events_refresh"];
    assert_eq!(refresh["unique"], true);
    for field in ["valid", "ready", "live"] {
        assert_eq!(refresh[field], true);
    }
    assert_eq!(refresh["keys"][0]["column"], "id");
    assert_eq!(indexes["recent_events_rank"]["keys"][0]["descending"], true);
    assert_eq!(
        complete["views"]["ordinary_events"]["indexes"],
        serde_json::json!({})
    );
    let ordering = generate(&PostgresCatalogOptions {
        coverage: PostgresCatalogCoverage::Ordering,
        ..options
    })
    .unwrap();
    assert!(ordering.get("views").is_none());
    assert_eq!(
        ordering["tables"]["events"]["indexes"],
        complete["tables"]["events"]["indexes"]
    );
}

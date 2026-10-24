use std::path::PathBuf;
fn fixture() -> PathBuf {
    crate::test_support::rule_fixture_root("postgres-no-offset").join("executor-metadata-low")
}
fn advisories(config: &str) -> Vec<serde_json::Value> {
    let root = fixture();
    let options = crate::napi_api::options::test_json_arg(serde_json::json!({
        "root": root, "config": root.join(config)
    }));
    let report: serde_json::Value =
        serde_json::from_str(&crate::napi_api::check_json_impl(options).unwrap()).unwrap();
    assert!(report["rules"].as_array().unwrap().is_empty(), "{report}");
    report["advisories"].as_array().unwrap().clone()
}
#[test]
fn executor_metadata_is_not_an_additional_sql_obligation() {
    let rows = advisories(".no-mistakes-low.yml");
    let source = std::fs::read_to_string(fixture().join("src/query.mts")).unwrap();
    for (index, line) in source.lines().enumerate() {
        let count = rows
            .iter()
            .filter(|row| row["file"] == "src/query.mts" && row["line"] == index + 1)
            .count();
        if line.contains("SQL is known") {
            assert_eq!(count, 0, "{rows:#?}");
        } else if line.contains("exactly the dynamic")
            || line.contains("Unknown helper")
            || line.contains("real unknown executor")
            || line.contains("// ADVISE:")
        {
            assert_eq!(count, 1, "line {}: {rows:#?}", index + 1);
        }
    }
}
#[test]
fn executor_metadata_exemptions_do_not_cross_profiles() {
    let rows = advisories(".no-mistakes-mixed.yml");
    assert_eq!(rows.len(), 1, "{rows:#?}");
    assert_eq!(rows[0]["file"], "src/mixed.mts");
    assert_eq!(rows[0]["line"], 5);
}

#[test]
fn library_bind_values_are_data_but_opaque_nested_uses_remain_unknown() {
    let rows = advisories(".no-mistakes-low.yml");
    for name in ["binds.mts", "bind-writes.mts"] {
        let file = format!("src/{name}");
        let source = std::fs::read_to_string(fixture().join(&file)).unwrap();
        for (index, line) in source.lines().enumerate() {
            let count = rows
                .iter()
                .filter(|row| row["file"] == file && row["line"] == index + 1)
                .count();
            if line.contains("// BIND_CLEAR") {
                assert_eq!(count, 0, "{rows:#?}");
            }
            if line.contains("// BIND_ESCAPE") {
                assert_eq!(count, 1, "{rows:#?}");
            }
            if line.contains("// BIND_UNKNOWN") {
                assert!(count > 0, "{rows:#?}");
            }
        }
    }
}

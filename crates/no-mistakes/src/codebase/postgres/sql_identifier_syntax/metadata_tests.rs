use std::path::PathBuf;
#[test]
fn callable_factory_executors_retain_values_builder_inputs() {
    let root =
        crate::test_support::rule_fixture_root("postgres-no-offset").join("low-values-executors");
    let source = std::fs::read_to_string(root.join("src/query.mts")).unwrap();
    let nested_sink_line = source
        .lines()
        .enumerate()
        .skip_while(|(_, line)| !line.starts_with("export async function nestedSqlArgument("))
        .find(|(_, line)| line.trim() == "return query(statement);")
        .map(|(index, _)| index + 1)
        .expect("saved nested factory case has an executor sink");
    let options = crate::napi_api::options::test_json_arg(serde_json::json!({
        "root": root,
    }));
    let report: serde_json::Value =
        serde_json::from_str(&crate::napi_api::check_json_impl(options).unwrap()).unwrap();
    let rows = report["advisories"].as_array().unwrap();
    for (rule, index) in [
        ("postgres-no-offset", 0),
        ("postgres-no-unbounded-sql-identifier", 1),
        ("postgres-no-mutable-sql-registry", 2),
    ] {
        let messages: Vec<_> = rows
            .iter()
            .filter(|row| row["rule"] == rule)
            .map(|row| row["message"].as_str().unwrap())
            .collect();
        assert!(!messages.is_empty(), "{rule}: {report}");
        let identity = format!("{rule} application #{index}");
        assert!(
            messages.iter().all(|message| message.contains(&identity)),
            "{identity}: {messages:?}"
        );
    }
    let guarded = rows
        .iter()
        .filter(|row| row["file"] == "src/guarded.mts")
        .collect::<Vec<_>>();
    assert!(
        guarded.iter().all(|row| row["line"].as_u64().unwrap() > 12),
        "{report}"
    );
    for line in [19, 26] {
        assert!(guarded.iter().any(|row| row["line"] == line), "{report}");
    }
    assert!(
        rows.iter().all(|row| row["file"] != "src/renamed.mts"),
        "{report}"
    );
    assert!(
        rows.iter().any(|row| row["file"] == "src/reassigned.mts"),
        "{report}"
    );
    let rows = rows
        .iter()
        .filter(|row| row["file"] == "src/query.mts")
        .collect::<Vec<_>>();
    for line in [4, 11, 27, 32, 38, 63] {
        assert!(rows.iter().all(|row| row["line"] != line), "{report}");
    }
    for line in [18, 45, 51, 57] {
        assert!(rows.iter().any(|row| row["line"] == line), "{report}");
    }
    for rule in [
        "postgres-no-offset",
        "postgres-no-unbounded-sql-identifier",
        "postgres-no-mutable-sql-registry",
    ] {
        assert_eq!(
            rows.iter()
                .filter(|row| row["line"] == nested_sink_line && row["rule"] == rule)
                .count(),
            1,
            "{rule} at nested sink: {report}"
        );
    }
    let findings = report["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 2, "{report}");
    for (file, line) in [("src/query.mts", 60), ("src/guarded.mts", 15)] {
        assert!(
            findings
                .iter()
                .any(|row| row["file"] == file && row["line"] == line),
            "{report}"
        );
    }
}
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
        if line.contains("SQL is known")
            || line.contains("INITIALIZER_CLEAR")
            || line.contains("SHADOW_CLEAR")
        {
            assert_eq!(count, 0, "{rows:#?}");
        } else if line.contains("exactly the dynamic")
            || line.contains("Unknown helper")
            || line.contains("real unknown executor")
            || line.contains("// ADVISE:")
        {
            assert_eq!(count, 1, "line {}: {rows:#?}", index + 1);
        } else if line.contains("// FALLBACK:") {
            assert!(count > 0, "{rows:#?}");
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

#[test]
fn runtime_data_spans_clear_only_at_proven_data_locations() {
    let root =
        crate::test_support::rule_fixture_root("postgres-no-offset").join("bind-runtime-spans");
    let options = crate::napi_api::options::test_json_arg(serde_json::json!({
        "root": root, "config": root.join(".no-mistakes-low.yml")
    }));
    let report: serde_json::Value =
        serde_json::from_str(&crate::napi_api::check_json_impl(options).unwrap()).unwrap();
    assert!(report["rules"].as_array().unwrap().is_empty(), "{report}");
    let rows = report["advisories"].as_array().unwrap();
    let source = std::fs::read_to_string(root.join("src/query.mts")).unwrap();
    let expected = source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| line.contains("// UNKNOWN:").then_some(index + 1))
        .collect::<Vec<_>>();
    for (index, line) in source.lines().enumerate() {
        let count = rows
            .iter()
            .filter(|row| row["file"] == "src/query.mts" && row["line"] == index + 1)
            .count();
        if line.contains("// CLEAR:") {
            assert_eq!(count, 0, "{report}");
        }
        if line.contains("// UNKNOWN:") {
            assert!(count > 0, "{report}");
        }
    }
    assert!(
        rows.iter()
            .all(|row| expected.contains(&(row["line"].as_u64().unwrap() as usize))),
        "{report}"
    );
}

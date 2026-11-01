use serde_json::{json, Value};

#[test]
fn typed_handoffs_clear_only_exact_receivers_and_keep_independent_sql_obligations() {
    let root = crate::test_support::rule_fixture_root("postgres-no-unbounded-sql-identifier")
        .join("low-typed-transports");
    let report: Value = serde_json::from_str(
        &crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(json!({
            "root": root, "config": root.join(".no-mistakes-low.yml")
        })))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(report["warnings"], json!([]));
    let source = std::fs::read_to_string(root.join("src/query.mts")).unwrap();
    for rule in [
        "postgres-no-unbounded-sql-identifier",
        "postgres-no-transformed-uuid-predicates",
    ] {
        let actual = report["advisories"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|row| row["rule"] == rule && row["file"] == "src/query.mts")
            .map(|row| row["line"].as_u64().unwrap() as usize)
            .collect::<Vec<_>>();
        let expected = source
            .lines()
            .enumerate()
            .filter_map(|(index, line)| {
                (line.contains("// UNKNOWN_")
                    || (rule.ends_with("uuid-predicates") && line.contains("// IDENTIFIER")))
                .then_some(index + 1)
            })
            .collect::<Vec<_>>();
        assert_eq!(actual, expected, "{rule}: {report}");
        assert!(
            report["advisories"].as_array().unwrap().iter().any(|row| {
                row["rule"] == rule && row["file"] == "src/receivers.mts" && row["line"] == 5
            }),
            "the callee's opaque executor escape remains independently unknown: {report}"
        );
    }
    let findings = report["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 3, "{report}");
    for (file, marker) in [("query.mts", "UUID_TAG"), ("receivers.mts", "UUID_CALLEE")] {
        let text = std::fs::read_to_string(root.join("src").join(file)).unwrap();
        let line = text.lines().position(|line| line.contains(marker)).unwrap() + 1;
        assert!(
            findings.iter().any(
                |row| row["rule"] == "postgres-no-transformed-uuid-predicates"
                    && row["file"] == format!("src/{file}")
                    && row["line"] == line
            ),
            "{report}"
        );
    }
}

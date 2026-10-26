#[test]
fn produced_receivers_stay_unknown_only_at_actual_consumers() {
    let root =
        crate::test_support::rule_fixture_root("postgres-no-offset").join("producer-consumers");
    let source = std::fs::read_to_string(root.join("src/query.ts")).unwrap();
    let report: serde_json::Value = serde_json::from_str(
        &crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
            serde_json::json!({ "root": root, "config": root.join(".no-mistakes-low.yml"),
                "includeSqlVerdicts": true }),
        ))
        .unwrap(),
    )
    .unwrap();
    assert!(
        report["warnings"].as_array().unwrap().is_empty(),
        "{report}"
    );
    assert!(report["rules"].as_array().unwrap().is_empty(), "{report}");
    let expected: Vec<_> = source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| line.contains("// UNKNOWN_").then_some(index + 1))
        .collect();
    for rule in [
        "postgres-no-offset",
        "postgres-no-unbounded-sql-identifier",
        "postgres-no-mutable-sql-registry",
        "postgres-no-transformed-uuid-predicates",
    ] {
        let actual: Vec<_> = report["advisories"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|row| row["rule"] == rule)
            .map(|row| row["line"].as_u64().unwrap() as usize)
            .collect();
        assert_eq!(actual, expected, "{rule}: {report}");
        let verdicts = report["sqlVerdicts"].as_array().unwrap();
        for row in verdicts
            .iter()
            .filter(|row| row["rule"] == rule && row["advisory"] == true)
        {
            assert_eq!(row["status"], "abstain", "{row}");
        }
    }
}

#[test]
fn asserted_configured_factories_preserve_actual_statement_proofs() {
    let root =
        crate::test_support::rule_fixture_root("postgres-no-offset").join("factory-cast-consumers");
    let source = std::fs::read_to_string(root.join("src/query.ts")).unwrap();
    let report: serde_json::Value = serde_json::from_str(
        &crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
            serde_json::json!({ "root": root, "config": root.join(".no-mistakes-low.yml"),
                "includeSqlVerdicts": true }),
        ))
        .unwrap(),
    )
    .unwrap();
    assert!(
        report["warnings"].as_array().unwrap().is_empty(),
        "{report}"
    );
    assert!(report["rules"].as_array().unwrap().is_empty(), "{report}");
    let lines = |marker| {
        source
            .lines()
            .enumerate()
            .filter_map(|(index, line)| line.contains(marker).then_some(index + 1))
            .collect::<Vec<_>>()
    };
    let clear = lines("// CLEAR_");
    let unknown = lines("// UNKNOWN_");
    assert_eq!((clear.len(), unknown.len()), (3, 3));
    for rule in [
        "postgres-no-offset",
        "postgres-no-unbounded-sql-identifier",
        "postgres-no-mutable-sql-registry",
        "postgres-no-transformed-uuid-predicates",
    ] {
        let actual: Vec<_> = report["advisories"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|row| row["rule"] == rule)
            .map(|row| row["line"].as_u64().unwrap() as usize)
            .collect();
        assert_eq!(actual, unknown, "{rule}: {report}");
        let verdicts = report["sqlVerdicts"].as_array().unwrap();
        for (expected, status) in [(&clear, "proven-clear"), (&unknown, "abstain")] {
            let actual: Vec<_> = verdicts
                .iter()
                .filter(|row| row["rule"] == rule && row["status"] == status)
                .map(|row| {
                    crate::codebase::ts_source::byte_offset_to_line(
                        &source,
                        row["span"][0].as_u64().unwrap() as usize,
                    ) as usize
                })
                .collect();
            assert_eq!(&actual, expected, "{rule}/{status}: {report}");
        }
    }
}

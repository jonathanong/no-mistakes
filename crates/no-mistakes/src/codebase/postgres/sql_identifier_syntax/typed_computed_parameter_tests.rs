use serde_json::{json, Value};

#[test]
fn computed_typed_parameters_keep_unknown_calls_without_value_import_fallback() {
    let root = crate::test_support::rule_fixture_root("postgres-no-offset")
        .join("computed-typed-parameters");
    let source = std::fs::read_to_string(root.join("src/query.mts")).unwrap();
    let unknown = source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| line.contains("// UNKNOWN_").then_some(index + 1))
        .collect::<Vec<_>>();
    let definite = source
        .lines()
        .position(|line| line.contains("// DEFINITE_OFFSET"))
        .unwrap()
        + 1;
    assert_eq!(unknown.len(), 5);
    for mode in ["report", "advise", "ignore"] {
        let report: Value = serde_json::from_str(
            &crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(json!({
                "root": root, "config": root.join(format!(".no-mistakes-{mode}.yml")),
                "includeSqlVerdicts": true
            })))
            .unwrap(),
        )
        .unwrap();
        assert!(
            report["warnings"].as_array().unwrap().is_empty(),
            "{report}"
        );
        for field in ["rules", "advisories"] {
            for row in report[field].as_array().unwrap() {
                assert_eq!(row["file"], "src/query.mts", "suppressed files: {row}");
            }
        }
        for rule in [
            "postgres-no-offset",
            "postgres-no-unbounded-sql-identifier",
            "postgres-no-mutable-sql-registry",
            "postgres-no-transformed-uuid-predicates",
        ] {
            let rows = report["rules"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|row| row["rule"] == rule)
                .collect::<Vec<_>>();
            let mut findings = rows
                .iter()
                .map(|row| row["line"].as_u64().unwrap() as usize)
                .collect::<Vec<_>>();
            findings.sort_unstable();
            let mut expected = if mode == "report" {
                unknown.clone()
            } else {
                Vec::new()
            };
            if rule == "postgres-no-offset" {
                expected.push(definite);
            }
            expected.sort_unstable();
            assert_eq!(findings, expected, "{mode} {rule}: {report}");
            let mut advice = report["advisories"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|row| row["rule"] == rule)
                .map(|row| row["line"].as_u64().unwrap() as usize)
                .collect::<Vec<_>>();
            advice.sort_unstable();
            assert_eq!(
                advice,
                if mode == "advise" {
                    unknown.clone()
                } else {
                    Vec::new()
                },
                "{mode} {rule}: {report}"
            );
            let abstained = report["sqlVerdicts"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|row| {
                    row["rule"] == rule
                        && row["file"] == "src/query.mts"
                        && row["status"] == "abstain"
                })
                .count();
            assert_eq!(
                abstained,
                unknown.len(),
                "ignore must retain uncertainty evidence: {report}"
            );
            if rule == "postgres-no-offset" {
                for row in rows {
                    if row["line"] != definite {
                        assert_eq!(row["target"], "unanalyzable", "{row}");
                    }
                }
            }
        }
    }
}

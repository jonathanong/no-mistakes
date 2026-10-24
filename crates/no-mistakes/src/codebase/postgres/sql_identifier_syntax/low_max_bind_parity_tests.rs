use serde_json::{json, Value};

#[test]
fn parameterized_low_results_do_not_borrow_max_sql_or_source_positions() {
    let root =
        crate::test_support::rule_fixture_root("postgres-no-offset").join("low-max-bind-parity");
    let run = |config: &str| {
        serde_json::from_str::<Value>(
            &crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(json!({
                "root": root, "config": root.join(config), "includeSqlVerdicts": true
            })))
            .unwrap(),
        )
        .unwrap()
    };
    let baseline = run(".no-mistakes-low.yml");
    let mixed = run(".no-mistakes-mixed.yml");
    for field in ["codebase", "integration", "queues", "react", "warnings"] {
        assert_eq!(baseline[field], mixed[field], "{field}");
    }
    let low_verdicts = |report: &Value| {
        report["sqlVerdicts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|row| {
                row["application"]["name"]
                    .as_str()
                    .unwrap()
                    .starts_with("low-")
            })
            .cloned()
            .collect::<Vec<_>>()
    };
    let expected = low_verdicts(&baseline);
    assert!(!expected.is_empty(), "{baseline}");
    for name in ["low-offset", "low-identifier", "low-registry", "low-uuid"] {
        assert!(
            expected
                .iter()
                .any(|row| row["application"]["name"] == name),
            "{name}"
        );
    }
    assert_eq!(low_verdicts(&mixed), expected);
    let low_advisories = |report: &Value| {
        report["advisories"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|row| {
                row["message"]
                    .as_str()
                    .unwrap()
                    .contains("application \"low-")
            })
            .cloned()
            .collect::<Vec<_>>()
    };
    assert_eq!(low_advisories(&mixed), low_advisories(&baseline));
    let source = std::fs::read_to_string(root.join("src/query.mts")).unwrap();
    let line = |marker: &str| {
        source
            .lines()
            .position(|line| line.contains(marker))
            .unwrap()
            + 1
    };
    for report in [&baseline, &mixed] {
        for rule in [
            "postgres-no-offset",
            "postgres-no-transformed-uuid-predicates",
        ] {
            assert!(
                low_advisories(report)
                    .iter()
                    .any(|row| { row["rule"] == rule && row["line"] == line("UNKNOWN_RELATION") }),
                "a low bind is still invalid in a relation position: {report}"
            );
        }
        for (rule, marker) in [
            ("postgres-no-offset", "BOUND_OFFSET"),
            ("postgres-no-offset", "ORDINARY_LITERAL"),
            ("postgres-no-offset", "MULTILINE_OFFSET"),
            ("postgres-no-offset", "LOCAL_TAG_OFFSET"),
            (
                "postgres-no-transformed-uuid-predicates",
                "TRANSFORMED_UUID",
            ),
        ] {
            assert!(
                report["rules"].as_array().unwrap().iter().any(|row| {
                    row["rule"] == rule
                        && row["file"] == "src/query.mts"
                        && row["line"] == line(marker)
                }),
                "physical finding {marker} missing: {report}"
            );
        }
    }
}

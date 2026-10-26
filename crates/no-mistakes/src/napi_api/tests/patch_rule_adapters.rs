use serde_json::{json, Value};

#[test]
fn recovered_rule_adapters_preserve_origins_suppression_and_invalid_alternatives() {
    let root =
        crate::test_support::rule_fixture_root("postgres-no-offset").join("low-helper-recovery");
    let source = std::fs::read_to_string(root.join("src/patch-rule-adapters.mts")).unwrap();
    let file = "src/patch-rule-adapters.mts";
    let rules = [
        "postgres-no-offset",
        "postgres-no-unbounded-sql-identifier",
        "postgres-no-mutable-sql-registry",
        "postgres-no-transformed-uuid-predicates",
    ];
    let uuid = rules[3];
    for (mode, config) in [
        ("advise", ".no-mistakes-low.yml"),
        ("report", ".no-mistakes-low-report.yml"),
        ("ignore", ".no-mistakes-low-ignore.yml"),
    ] {
        let report: Value = serde_json::from_str(
            &crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(json!({
                "root": root, "config": root.join(config), "includeSqlVerdicts": true,
            })))
            .unwrap(),
        )
        .unwrap();
        assert!(
            report["warnings"].as_array().unwrap().is_empty(),
            "{report}"
        );
        let has = |channel: &str, rule: &str, marker: &str| {
            report[channel].as_array().unwrap().iter().any(|row| {
                row["rule"] == rule
                    && row["file"] == file
                    && row["line"] == adapter_line(&source, marker) as u64
            })
        };
        for marker in ["UUID_UNKNOWN_TABLE", "UUID_UNKNOWN_COLUMN"] {
            assert_eq!(
                has("advisories", uuid, marker),
                mode == "advise",
                "{report}"
            );
            assert_eq!(has("rules", uuid, marker), mode == "report", "{report}");
        }
        for marker in [
            "UUID_TEXT_CLEAR",
            "SUPPRESSED_UUID_CALL",
            "SUPPRESSED_UUID_ORIGIN",
        ] {
            assert!(!has("rules", uuid, marker), "{report}");
            assert!(!has("advisories", uuid, marker), "{report}");
        }
        assert!(has("rules", uuid, "UUID_MIX_ORIGIN"), "{mode}: {report}");
        assert!(
            has("rules", rules[0], "OFFSET_MIX_ORIGIN"),
            "{mode}: {report}"
        );
        assert!(
            has("rules", rules[0], "MULTILINE_OFFSET_ORIGIN"),
            "{mode}: {report}"
        );
        for marker in [
            "UUID_MIX_CALL",
            "OFFSET_MIX_CALL",
            "INVALID_QUOTED_BIND",
            "INVALID_TOKENIZER_BIND",
        ] {
            for rule in rules {
                assert_eq!(
                    has("advisories", rule, marker),
                    mode == "advise",
                    "{report}"
                );
                assert_eq!(has("rules", rule, marker), mode == "report", "{report}");
            }
        }
        let verdicts = report["sqlVerdicts"].as_array().unwrap();
        for rule in rules {
            let clear = verdicts.iter().any(|row| {
                row["rule"] == rule
                    && row["file"] == file
                    && adapter_span_line(&source, row) == adapter_line(&source, "UUID_TEXT_CLEAR")
                    && row["status"] == "proven-clear"
            });
            assert!(clear, "{mode}: missing text-column clear: {report}");
        }
        for (rule, marker) in [(uuid, "UUID_MIX_CALL"), (rules[0], "OFFSET_MIX_CALL")] {
            assert!(
                verdicts.iter().any(|row| {
                    row["rule"] == rule
                        && row["file"] == file
                        && adapter_span_line(&source, row) == adapter_line(&source, marker)
                        && row["status"] == "violation"
                }),
                "{mode}: definite finding lost: {report}"
            );
        }
    }
}

fn adapter_line(source: &str, marker: &str) -> usize {
    source
        .lines()
        .position(|line| line.contains(marker))
        .unwrap()
        + 1
}

fn adapter_span_line(source: &str, row: &Value) -> usize {
    crate::codebase::ts_source::byte_offset_to_line(
        source,
        row["span"][0].as_u64().unwrap() as usize,
    ) as usize
}

use super::check_json_snapshot::rules_path;
use serde_json::Value;
use std::process::Command;

fn report(config: &str) -> Value {
    let root = rules_path("postgres-no-transformed-uuid-predicates", "fixture", ".");
    let config = rules_path("postgres-no-transformed-uuid-predicates", "configs", config);
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--root"])
        .arg(root)
        .arg("--config")
        .arg(config)
        .args(["--format", "json"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn uuid_transforms_report_each_expression_once_with_all_columns_at_both_efforts() {
    for config in ["multi-argument-sites.yml", "multi-argument-sites-low.yml"] {
        let report = report(config);
        let findings = report["rules"].as_array().unwrap();
        let sql: Vec<_> = findings
            .iter()
            .filter(|finding| finding["file"] == "multi-argument-sites.sql")
            .collect();
        let lines: Vec<_> = sql
            .iter()
            .map(|finding| finding["line"].as_u64().unwrap())
            .collect();
        assert_eq!(
            lines,
            [1, 2, 3, 4, 4, 6, 9, 10, 10, 11, 11, 15, 19],
            "{report}"
        );
        for (line, expected) in [(1, 2), (2, 4), (3, 4), (6, 2)] {
            let finding = sql.iter().find(|finding| finding["line"] == line).unwrap();
            let columns: Vec<_> = finding["target"].as_str().unwrap().split(", ").collect();
            assert_eq!(columns.len(), expected, "{finding}");
            for column in columns {
                assert!(finding["message"].as_str().unwrap().contains(column));
            }
        }
        let repeated: Vec<_> = sql.iter().filter(|finding| finding["line"] == 4).collect();
        assert_ne!(repeated[0]["message"], repeated[1]["message"]);
        let ts: Vec<_> = findings
            .iter()
            .filter(|finding| finding["file"] == "multi-argument-sites.ts")
            .collect();
        assert_eq!(ts.len(), 4, "{report}");
        assert!(ts.iter().all(|finding| finding["line"] != 10));
        assert!(ts
            .iter()
            .all(|finding| finding["target"] == "accounts.id, accounts.token"));
    }
}

#[test]
fn uuid_variants_union_affected_columns_at_one_physical_expression() {
    for config in [
        "multi-argument-variants.yml",
        "multi-argument-variants-low.yml",
        "multi-argument-shared-helper.yml",
        "multi-argument-shared-helper-low.yml",
    ] {
        let report = report(config);
        let findings = report["rules"].as_array().unwrap();
        assert_eq!(findings.len(), 1, "{report}");
        assert_eq!(findings[0]["target"], "accounts.id, accounts.token");
    }
}

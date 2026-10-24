use std::path::PathBuf;
use std::process::Command;

#[test]
fn procedural_bodies_are_skipped_but_unknown_sql_and_top_level_siblings_report() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-ddl-unanalyzable/fixture/procedural");
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--format", "json", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(root.join("report.yml"))
        .output()
        .unwrap();
    assert!(!output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let findings = report["rules"].as_array().unwrap();
    let mut actual = findings
        .iter()
        .map(|finding| {
            (
                finding["file"].as_str().unwrap(),
                finding["line"].as_u64().unwrap(),
            )
        })
        .collect::<Vec<_>>();
    actual.sort_unstable();
    assert_eq!(
        actual,
        [
            ("migrations/block-command-followups.sql", 11),
            ("migrations/block-command-followups.sql", 15),
            ("migrations/block-command-followups.sql", 19),
            ("migrations/block-command-followups.sql", 25),
            ("migrations/block-command-followups.sql", 28),
            ("migrations/block-command-followups.sql", 39),
            ("migrations/block-command-followups.sql", 43),
            ("migrations/block-command-followups.sql", 47),
            ("migrations/block-command-followups.sql", 53),
            ("migrations/block-command-followups.sql", 56),
            ("migrations/command-followups.sql", 11),
            ("migrations/command-followups.sql", 20),
            ("migrations/grammar-errors.sql", 4),
            ("migrations/grammar-errors.sql", 6),
            ("migrations/grammar-errors.sql", 7),
            ("migrations/grammar-errors.sql", 8),
            ("migrations/grammar-errors.sql", 9),
            ("migrations/grammar-errors.sql", 14),
            ("migrations/grammar-errors.sql", 15),
            ("migrations/grammar-errors.sql", 16),
            ("migrations/grammar-errors.sql", 17),
            ("migrations/grammar-errors.sql", 18),
            ("migrations/grammar-errors.sql", 19),
            ("migrations/grammar-errors.sql", 20),
            ("migrations/grammar-errors.sql", 21),
            ("migrations/grammar-errors.sql", 24),
            ("migrations/grammar-errors.sql", 27),
            ("migrations/grammar-errors.sql", 30),
            ("migrations/grammar-errors.sql", 33),
            ("migrations/grammar-errors.sql", 36),
            ("migrations/grammar-errors.sql", 44),
            ("migrations/grammar-errors.sql", 46),
            ("migrations/grammar-errors.sql", 47),
            ("migrations/grammar-errors.sql", 49),
            ("migrations/grammar-errors.sql", 50),
            ("migrations/grammar-errors.sql", 51),
            ("migrations/grammar-errors.sql", 52),
            ("migrations/grammar-errors.sql", 53),
            ("migrations/grammar-errors.sql", 54),
            ("migrations/grammar-errors.sql", 55),
            ("migrations/grammar-errors.sql", 57),
            ("migrations/grammar-errors.sql", 58),
            ("migrations/lexical-command-error.sql", 2),
            ("migrations/lexical-command-error.sql", 10),
            ("migrations/lexical-command-error.sql", 13),
            ("migrations/lexical-command-error.sql", 19),
            ("migrations/malformed-command-tail.sql", 7),
            ("migrations/malformed-do-suffix.sql", 2),
            ("migrations/malformed-routine-headers.sql", 2),
            ("migrations/malformed-routine-headers.sql", 8),
            ("migrations/return-query.sql", 3),
            ("migrations/top-level-shapes.sql", 2),
            ("migrations/top-level-shapes.sql", 3),
            ("migrations/top-level.sql", 7),
            ("migrations/top-level.sql", 13),
            ("migrations/top-level.sql", 19),
            ("migrations/top-level.sql", 21),
            ("migrations/unknown-loop-query.sql", 6),
            ("migrations/unknown-payloads.sql", 7),
            ("migrations/unknown-payloads.sql", 8),
            ("migrations/unknown-payloads.sql", 9),
            ("migrations/unknown-payloads.sql", 10),
            ("migrations/unknown-payloads.sql", 11),
            ("migrations/unknown-procedural-commands.sql", 4),
            ("migrations/unknown-procedural-commands.sql", 10),
            ("migrations/unknown-procedural-commands.sql", 15),
            ("migrations/unknown.sql", 6),
            ("migrations/unknown.sql", 14),
            ("migrations/unknown.sql", 22),
        ],
        "{report}"
    );
}

#[test]
fn original_heads_and_rejected_branch_locations_survive_cli_preparation() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-ddl-unanalyzable/fixture/issue-2731");
    let expected: std::collections::BTreeMap<String, Vec<(usize, usize, String)>> =
        serde_json::from_str(&std::fs::read_to_string(root.join("expected.json")).unwrap())
            .unwrap();
    let mut expected: Vec<_> = expected
        .into_iter()
        .flat_map(|(name, rows)| {
            rows.into_iter()
                .map(move |(line, _, head)| (format!("migrations/{name}"), line, head))
        })
        .collect();
    expected.sort_unstable();
    for config in ["report.yml", "low.yml"] {
        let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
            .args(["check", "--format", "json", "--root"])
            .arg(&root)
            .arg("--config")
            .arg(root.join(config))
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1));
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        let warnings = report["warnings"].as_array().unwrap();
        assert_eq!(warnings.len(), if config == "low.yml" { 11 } else { 0 });
        assert!(warnings.iter().all(|warning| {
            warning.as_str().unwrap().ends_with(
            "requested effort low; effective effort max because this rule has no low implementation"
        )
        }));

        assert_eq!(report["advisories"], serde_json::json!([]));
        let findings = report["rules"].as_array().unwrap();
        // Two existing CREATE VIEW policy violations accompany the 35 failures per schema rule.
        assert_eq!(findings.len(), 387, "{report}");
        let mut by_rule = std::collections::BTreeMap::<String, Vec<_>>::new();
        for finding in findings
            .iter()
            .filter(|finding| finding["target"] != "CREATE VIEW")
        {
            let head = finding["target"].as_str().unwrap();
            assert!(finding["message"]
                .as_str()
                .unwrap()
                .contains(&format!("SQL statement `{head}` could not be analyzed")));
            by_rule
                .entry(finding["rule"].as_str().unwrap().to_owned())
                .or_default()
                .push((
                    finding["file"].as_str().unwrap().to_owned(),
                    finding["line"].as_u64().unwrap() as usize,
                    head.to_owned(),
                ));
        }
        assert_eq!(by_rule.len(), 11);
        for (rule, mut findings) in by_rule {
            findings.sort_unstable();
            assert_eq!(findings, expected, "{config}: {rule}");
        }
    }
}

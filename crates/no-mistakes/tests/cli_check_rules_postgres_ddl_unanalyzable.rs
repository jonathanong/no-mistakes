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
                finding["target"].as_str().unwrap(),
            )
        })
        .collect::<Vec<_>>();
    actual.sort_unstable();
    assert_eq!(
        actual,
        [
            ("migrations/malformed-do-suffix.sql", 2, "unanalyzable"),
            ("migrations/return-query.sql", 3, "unanalyzable"),
            ("migrations/top-level.sql", 7, "unanalyzable"),
            ("migrations/top-level.sql", 13, "unanalyzable"),
            ("migrations/top-level.sql", 19, "unanalyzable"),
            ("migrations/top-level.sql", 21, "unanalyzable"),
            ("migrations/unknown.sql", 6, "unanalyzable"),
            ("migrations/unknown.sql", 14, "unanalyzable"),
            ("migrations/unknown.sql", 22, "unanalyzable"),
        ],
        "{report}"
    );
}

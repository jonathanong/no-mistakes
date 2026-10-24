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
            ("migrations/malformed-do-suffix.sql", 2),
            ("migrations/malformed-routine-headers.sql", 2,),
            ("migrations/malformed-routine-headers.sql", 8,),
            ("migrations/return-query.sql", 3),
            ("migrations/top-level.sql", 7),
            ("migrations/top-level.sql", 13),
            ("migrations/top-level.sql", 19),
            ("migrations/top-level.sql", 21),
            ("migrations/unknown-procedural-commands.sql", 4,),
            ("migrations/unknown-procedural-commands.sql", 10,),
            ("migrations/unknown-procedural-commands.sql", 15,),
            ("migrations/unknown.sql", 6),
            ("migrations/unknown.sql", 14),
            ("migrations/unknown.sql", 22),
        ],
        "{report}"
    );
}

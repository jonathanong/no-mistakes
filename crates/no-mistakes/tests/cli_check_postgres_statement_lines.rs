use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn repeated_foreign_keys_use_their_own_statements_and_suppressions() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-fk-index/fixture/statement-spans");
    let mut baseline = None;
    for jobs in ["1", "4", "0"] {
        let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
            .args(["check", "--format", "json", "--root"])
            .arg(&root)
            .args(["--jobs", jobs])
            .output()
            .unwrap();
        assert!(!output.status.success());
        let report: Value = serde_json::from_slice(&output.stdout).unwrap();
        if let Some(previous) = &baseline {
            assert_eq!(&report, previous, "jobs {jobs}");
        }
        let actual: Vec<_> = report["rules"]
            .as_array()
            .unwrap()
            .iter()
            .map(|finding| {
                (
                    finding["file"].as_str().unwrap(),
                    finding["line"].as_u64().unwrap(),
                    finding["target"].as_str().unwrap(),
                )
            })
            .collect();
        assert_eq!(
            actual,
            [
                ("later-allow.sql", 1, "AllowFirst.personId"),
                ("later-disable.sql", 1, "First.personId"),
                ("multiline.sql", 2, "MultilineFirst.personId"),
                ("multiline.sql", 5, "MultilineSecond.personId"),
                ("repeated.sql", 1, "Attribute.personId"),
                ("repeated.sql", 2, "Session.personId"),
            ],
            "{report}"
        );
        baseline = Some(report);
    }
}

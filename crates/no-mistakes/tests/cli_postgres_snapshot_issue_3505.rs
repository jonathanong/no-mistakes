use super::check_json_clean::assert_saved_fixture_clean;
use super::check_json_snapshot::{assert_saved_fixture_report, rules_path};
use std::process::Command;

#[test]
fn postgres_identifier_parameterized_fragments_check_json_snapshot() {
    let root = rules_path(
        "postgres-no-unbounded-sql-identifier",
        "fixture",
        "issue-3505",
    );
    assert_saved_fixture_clean(
        "postgres_identifier_parameterized_fragments",
        &[
            ("max", root.clone(), root.join(".no-mistakes.yml")),
            (
                "project",
                root.clone(),
                root.join(".no-mistakes-project-valid.yml"),
            ),
        ],
    );
    let negative = assert_saved_fixture_report(
        "postgres_identifier_parameterized_fragments_negative",
        &[("max", root.clone(), root.join(".no-mistakes-negative.yml"))],
        &[],
        &[],
    );
    let findings = negative["max"]["report"]["rules"].as_array().unwrap();
    for (marker, target) in [
        ("reassigned", "q"),
        ("missing values", "q"),
        ("wrong values", "q"),
        ("helper values", "q"),
        ("append values", "q"),
        ("imported values", "imported"),
        ("recursive", "q"),
        ("namespace values", "statements.namespaceStatement"),
        ("chained append", "q"),
        ("multi missing", "q"),
        ("multi wrong", "q"),
    ] {
        assert!(
            findings.iter().any(|finding| {
                finding["target"] == target
                    && finding["sourceLine"]
                        .as_str()
                        .is_some_and(|line| line.contains(marker))
            }),
            "missing {marker}: {negative}"
        );
    }
    for jobs in ["1", "2"] {
        let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
            .args(["check", "--root"])
            .arg(&root)
            .arg("--config")
            .arg(root.join(".no-mistakes-project.yml"))
            .args(["--jobs", jobs, "--format", "json"])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2));
        assert!(String::from_utf8_lossy(&output.stderr)
            .contains("options.trustedSqlTags restates the postgres project scope"));
    }
}

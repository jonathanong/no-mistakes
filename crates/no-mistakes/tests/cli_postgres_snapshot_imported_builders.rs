use super::check_json_clean::assert_saved_fixture_clean;
use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::process::Command;

#[test]
fn postgres_identifier_imported_builders_check_json_snapshot() {
    let root = rules_path(
        "postgres-no-unbounded-sql-identifier",
        "fixture",
        "issue-3073-imported-builders",
    );
    assert_saved_fixture_clean(
        "postgres_identifier_imported_builders_clean",
        &[("max", root.clone(), root.join(".no-mistakes.yml"))],
    );
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(root.join(".no-mistakes-negative.yml"))
        .args(["--format", "json"])
        .output()
        .unwrap();
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(
        report["rules"].as_array().unwrap().iter().any(|finding| {
            finding["file"] == "src/dirty.ts" && finding["line"] == 9 && finding["target"] == "raw"
        }),
        "typed executor callback must retain raw append finding: {report}"
    );
    assert_saved_fixture(
        "postgres_identifier_imported_builders_negative",
        &[("max", root.clone(), root.join(".no-mistakes-negative.yml"))],
    );
}

#[test]
fn postgres_identifier_builder_runtime_consumers_check_json_snapshot() {
    let fixtures = rules_path(
        "postgres-no-unbounded-sql-identifier",
        "fixture",
        "builder-runtime-consumers",
    );
    let cases: Vec<_> = ["dynamic", "conditional", "clean"]
        .into_iter()
        .map(|scenario| {
            let root = fixtures.join(scenario);
            let config = root.join(".no-mistakes.yml");
            (scenario, root, config)
        })
        .collect();
    assert_saved_fixture("postgres_identifier_builder_runtime_consumers", &cases);
}

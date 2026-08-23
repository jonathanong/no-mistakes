use std::path::PathBuf;
use std::process::{Command, Output};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn fixture() -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-no-generated-column-writes/fixture"),
    )
}

fn check_fixture_config(root: &PathBuf, name: &str) -> Output {
    let yaml = std::fs::read_to_string(root.join(name)).unwrap();
    let config = tempfile::Builder::new().suffix(".yml").tempfile().unwrap();
    std::fs::write(config.path(), yaml).unwrap();
    Command::new(bin())
        .args(["check", "--root"])
        .arg(root)
        .arg("--config")
        .arg(config.path())
        .output()
        .unwrap()
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn postgres_no_generated_column_writes_fails_for_each_dml_shape() {
    let root = fixture();
    let out = check_fixture_config(&root, ".no-mistakes.yml");
    let body = stdout(&out);

    assert!(!out.status.success(), "expected exit 1");
    assert!(
        body.contains("postgres-no-generated-column-writes"),
        "{body}"
    );
    assert!(body.contains("fail-update.ts"), "{body}");
    assert!(body.contains("fail-insert-cols.ts"), "{body}");
    assert!(body.contains("fail-insert-columnless.ts"), "{body}");
    assert!(body.contains("fail-upsert.ts"), "{body}");
    assert!(body.contains("fail-merge.sql"), "{body}");
    assert!(!body.contains("pass.ts"), "{body}");
}

#[test]
fn postgres_no_generated_column_writes_json_has_rule_id() {
    let root = fixture();
    let config = tempfile::Builder::new().suffix(".yml").tempfile().unwrap();
    std::fs::write(
        config.path(),
        "rules:\n  - rule: postgres-no-generated-column-writes\n    scope: repository\n",
    )
    .unwrap();
    let out = Command::new(bin())
        .args(["check", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(config.path())
        .args(["--format", "json"])
        .output()
        .unwrap();
    let body = stdout(&out);
    assert!(
        body.contains("postgres-no-generated-column-writes"),
        "{body}"
    );
    assert!(!out.status.success());
}

#[test]
fn postgres_no_generated_column_writes_filesystem_runner_discovers_files() {
    let root = fixture();
    let findings = no_mistakes::codebase::rules::run_filesystem_rules(&root, None).unwrap();
    let body = format!("{findings:?}");

    assert!(!findings.is_empty(), "expected findings");
    assert!(
        body.contains("postgres-no-generated-column-writes"),
        "{body}"
    );
    assert!(
        findings.iter().any(|finding| {
            finding.rule == no_mistakes::codebase::rules::POSTGRES_NO_GENERATED_COLUMN_WRITES
        }),
        "{body}"
    );
    assert!(
        !findings.iter().any(|finding| finding.file == "pass.ts"),
        "{body}"
    );
}

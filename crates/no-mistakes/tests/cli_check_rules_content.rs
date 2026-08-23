use std::path::PathBuf;
use std::process::{Command, Output};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn fixture(category: &str, scenario: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules")
            .join(category)
            .join("fixture")
            .join(scenario),
    )
}

fn check_fixture_config(root: &PathBuf, name: &str) -> Output {
    Command::new(bin())
        .args(["check", "--root"])
        .arg(root)
        .arg("--config")
        .arg(root.join(name))
        .output()
        .unwrap()
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn no_empty_or_comments_only_files_fails_for_comment_only_fixture() {
    let root = fixture("no-empty-or-comments-only-files", "fail");
    let findings = no_mistakes::codebase::rules::run_filesystem_rules(&root, None).unwrap();
    let body = format!("{findings:?}");

    assert!(!findings.is_empty(), "expected findings");
    assert!(body.contains("no-empty-or-comments-only-files"), "{body}");
    assert!(body.contains("placeholder.ts"), "{body}");
}

#[test]
fn no_empty_or_comments_only_files_cli_fails_for_comment_only_fixture() {
    let root = fixture("no-empty-or-comments-only-files", "fail");
    let output = check_fixture_config(&root, ".no-mistakes.yml");
    let body = stdout(&output);

    assert!(!output.status.success(), "expected exit 1");
    assert!(body.contains("no-empty-or-comments-only-files"), "{body}");
    assert!(body.contains("placeholder.ts"), "{body}");
}

#[test]
fn package_json_registry_only_fails_for_non_registry_dependency() {
    let root = fixture("package-json-registry-only", "fail");
    let findings = no_mistakes::codebase::rules::run_filesystem_rules(&root, None).unwrap();
    let body = format!("{findings:?}");

    assert!(!findings.is_empty(), "expected findings");
    assert!(body.contains("package-json-registry-only"), "{body}");
    assert!(body.contains("package.json"), "{body}");
}

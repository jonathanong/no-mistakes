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

fn check_fixture_config(root: &PathBuf) -> Output {
    Command::new(bin())
        .args(["check", "--root"])
        .arg(root)
        .arg("--config")
        .arg(root.join(".no-mistakes.yml"))
        .output()
        .unwrap()
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn markdown_child_links_fails_when_a_child_is_unlisted() {
    let root = fixture("markdown-child-links", "fail");
    let out = check_fixture_config(&root);
    let body = stdout(&out);
    assert!(!out.status.success(), "expected exit 1: {body}");
    assert!(body.contains("markdown-child-links"), "{body}");
    assert!(body.contains("guide.md"), "{body}");
}

#[test]
fn markdown_child_links_passes_with_a_whole_file_link() {
    let root = fixture("markdown-child-links", "pass");
    let out = check_fixture_config(&root);
    assert!(out.status.success(), "exit non-zero: {}", stdout(&out));
}

#[test]
fn markdown_child_links_counts_canonical_html_list_items() {
    let root = fixture("markdown-child-links", "canonical-html-pass");
    let out = check_fixture_config(&root);
    assert!(out.status.success(), "exit non-zero: {}", stdout(&out));
}

#[test]
fn markdown_child_links_rejects_canonical_html_fragment_only() {
    let root = fixture("markdown-child-links", "canonical-html-fragment");
    let out = check_fixture_config(&root);
    let body = stdout(&out);
    assert!(!out.status.success(), "expected exit 1: {body}");
    assert!(body.contains("markdown-child-links"), "{body}");
}

#[test]
fn markdown_eval_tests_fails_for_eval_spawn_heuristic() {
    let root = fixture("markdown-eval-tests", "fail");
    let out = check_fixture_config(&root);
    let body = stdout(&out);
    assert!(!out.status.success(), "expected exit 1: {body}");
    assert!(body.contains("markdown-eval-tests"), "{body}");
}

#[test]
fn markdown_eval_tests_passes_for_spawn_free_readers() {
    let root = fixture("markdown-eval-tests", "pass");
    let out = check_fixture_config(&root);
    assert!(out.status.success(), "exit non-zero: {}", stdout(&out));
}

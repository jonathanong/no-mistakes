use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/swift-viewmodel-main-actor/fixture")
            .join(name),
    )
}

fn config(yaml: &str) -> NoMistakesConfig {
    NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(yaml).unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    }
}

fn run(root: &Path, yaml: &str) -> Vec<RuleFinding> {
    let files = crate::codebase::ts_source::discover_files(root, &[]);
    check_with_files(root, &config(yaml), &files).unwrap()
}

fn run_deferred(root: &Path, yaml: &str, defer: bool) -> Vec<RuleFinding> {
    let files = crate::codebase::ts_source::discover_files(root, &[]);
    let sources = super::super::source_store_for_files(&files);
    check_with_files_sources_and_deferred_suppression(root, &config(yaml), &files, &sources, defer)
        .unwrap()
}

#[test]
fn flags_classes_missing_main_actor() {
    let findings = run(&fixture("fail"), "{}");
    let files: Vec<_> = findings.iter().map(|f| f.file.as_str()).collect();
    assert!(files.contains(&"BrokenViewModel.swift"), "{findings:?}");
    assert!(files.contains(&"BareViewModel.swift"), "{findings:?}");
    assert!(
        findings.iter().all(|f| f.message == DEFAULT_MESSAGE),
        "{findings:?}"
    );
}

#[test]
fn accepts_annotated_structs_actors_and_extensions() {
    assert!(run(&fixture("pass"), "{}").is_empty());
}

#[test]
fn allow_globs_skip_files() {
    assert!(run(&fixture("fail"), "allow: [\"BrokenViewModel.swift\"]")
        .iter()
        .all(|f| f.file != "BrokenViewModel.swift"));
}

#[test]
fn ignore_non_swift_and_comments() {
    let findings = run(&fixture("fail"), "{}");
    assert!(
        findings.iter().all(|f| f.file.ends_with(".swift")),
        "{findings:?}"
    );
}

#[test]
fn disable_comment_and_custom_message() {
    assert!(run(&fixture("disabled"), "{}").is_empty());
    let findings = run(&fixture("fail"), "message: extra");
    assert!(
        findings
            .iter()
            .any(|f| f.message == format!("{DEFAULT_MESSAGE} extra")),
        "{findings:?}"
    );
    let empty = run(
        &fixture("fail"),
        "message: \"\"\nsuffix: \"\"\nattribute: \"\"",
    );
    assert!(
        empty.iter().all(|f| f.message == DEFAULT_MESSAGE),
        "{empty:?}"
    );
}

#[test]
fn deferred_suppression_keeps_disabled_file() {
    let findings = run_deferred(&fixture("disabled"), "{}", true);
    assert!(
        findings
            .iter()
            .any(|f| f.file.ends_with("DisabledViewModel.swift")),
        "{findings:?}"
    );
}

#[test]
fn custom_suffix_and_attribute() {
    let findings = run(&fixture("custom"), "suffix: Presenter\nattribute: UIActor");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].file, "Presenters.swift");
    assert_eq!(findings[0].line, 6);
}

#[test]
fn invalid_allow_glob_errors() {
    let root = fixture("fail");
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let error = check_with_files(&root, &config("allow: [\"[\"]"), &files).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("swift-viewmodel-main-actor allow"),
        "{error:#}"
    );
}

#[test]
fn skips_missing_source() {
    let root = fixture("fail");
    let missing = root.join("missing.swift");
    let findings = check_with_files(&root, &config("{}"), &[missing]).unwrap();
    assert!(findings.is_empty(), "{findings:?}");
}

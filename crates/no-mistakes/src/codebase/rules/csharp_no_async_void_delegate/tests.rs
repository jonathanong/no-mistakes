use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/csharp-no-async-void-delegate/fixture")
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
fn flags_async_command_and_begin_invoke() {
    let findings = run(&fixture("fail"), "{}");
    let files: Vec<_> = findings.iter().map(|f| f.file.as_str()).collect();
    assert!(
        files.contains(&"Pages/LoadablePageBinding.cs"),
        "{findings:?}"
    );
    assert!(files.contains(&"Pages/TypedCommandHost.cs"), "{findings:?}");
    assert!(files.contains(&"Pages/SessionBinding.cs"), "{findings:?}");
    assert!(files.contains(&"Pages/MauiCommandHost.cs"), "{findings:?}");
    assert!(
        findings.iter().all(|f| f.message == DEFAULT_MESSAGE),
        "{findings:?}"
    );
}

#[test]
fn accepts_fire_and_forget_wraps() {
    assert!(run(&fixture("pass"), "{}").is_empty());
}

#[test]
fn allow_globs_skip_files() {
    assert!(
        run(&fixture("fail"), "allow: [\"Pages/SessionBinding.cs\"]")
            .iter()
            .all(|f| f.file != "Pages/SessionBinding.cs")
    );
}

#[test]
fn ignore_non_csharp_and_comments() {
    let findings = run(&fixture("fail"), "{}");
    assert!(
        findings.iter().all(|f| f.file.ends_with(".cs")),
        "{findings:?}"
    );
}

#[test]
fn disable_comment_and_custom_message() {
    let findings = run(&fixture("disabled"), "{}");
    assert!(findings.is_empty(), "{findings:?}");
    let findings = run(&fixture("fail"), "message: extra");
    assert!(
        findings
            .iter()
            .any(|f| f.message == format!("{DEFAULT_MESSAGE} extra")),
        "{findings:?}"
    );
}

#[test]
fn deferred_suppression_keeps_disabled_file() {
    let findings = run_deferred(&fixture("disabled"), "{}", true);
    assert!(
        findings.iter().any(|f| f.file.ends_with("Disabled.cs")),
        "{findings:?}"
    );
}

#[test]
fn custom_names_and_empty_message() {
    let findings = run(
        &fixture("custom"),
        "constructors: [RelayCommand]\nmethods: [DispatchAsync]",
    );
    assert_eq!(findings.len(), 2, "{findings:?}");
    let none = run(&fixture("fail"), "constructors: [\"\"]\nmethods: [\"\"]");
    assert!(none.is_empty(), "{none:?}");
}

#[test]
fn invalid_allow_glob_errors() {
    let root = fixture("fail");
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let error = check_with_files(&root, &config("allow: [\"[\"]"), &files).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("csharp-no-async-void-delegate allow"),
        "{error:#}"
    );
}

#[test]
fn skips_missing_source() {
    let root = fixture("fail");
    let missing = root.join("missing.cs");
    let findings = check_with_files(&root, &config("{}"), &[missing]).unwrap();
    assert!(findings.is_empty(), "{findings:?}");
}

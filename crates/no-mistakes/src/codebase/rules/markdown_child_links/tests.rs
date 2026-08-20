use super::*;
use crate::codebase::ts_source::FrozenPathRemapper;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/markdown-child-links/fixture")
            .join(name),
    )
}

fn config(require_whole_file: bool) -> NoMistakesConfig {
    NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(&format!(
                r#"
groups:
  - parents: ["docs/README.md"]
    children: ["docs/*.md"]
    requireWholeFile: {require_whole_file}
"#
            ))
            .unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    }
}

fn run(root: &Path, require_whole_file: bool) -> Vec<RuleFinding> {
    let files = vec![root.join("docs/README.md"), root.join("docs/guide.md")];
    let sources = super::super::source_store_for_files(&files);
    let mut plan = super::super::markdown_facts::MarkdownFactPlan::default();
    plan.request_pulldown(super::super::markdown_scope::markdown_files(&files));
    let facts = super::super::markdown_facts::MarkdownFactMap::prepare(&plan, &sources);
    check_with_files_sources_and_facts(root, &config(require_whole_file), &files, &facts).unwrap()
}

#[test]
fn missing_child_link_is_a_finding() {
    let findings = run(&fixture("fail"), true);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].file, "docs/guide.md");
    assert!(findings[0].message.contains("not linked"));
}

#[test]
fn whole_file_link_covers_the_child() {
    assert!(run(&fixture("pass"), true).is_empty());
}

#[test]
fn fragment_only_link_is_ignored_when_require_whole_file() {
    let findings = run(&fixture("fragment"), true);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("whole-file"));
}

#[test]
fn fragment_link_counts_when_whole_file_is_not_required() {
    assert!(run(&fixture("fragment"), false).is_empty());
}

#[test]
fn missing_child_link_without_whole_file_requirement() {
    let findings = run(&fixture("fail"), false);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(
        findings[0].message.contains("markdown link"),
        "{findings:?}"
    );
    assert!(!findings[0].message.contains("whole-file"), "{findings:?}");
}

#[test]
fn parent_link_resolution_covers_external_absolute_and_dot_paths() {
    let root = fixture("pass");
    let readme = root.join("docs/README.md");
    let guide = root.join("docs/guide.md");
    let known = BTreeSet::from([readme.clone(), guide.clone()]);
    let remapper = FrozenPathRemapper::from_paths([readme.clone(), guide.clone()]);
    let links = super::links::resolve_parent_links(
        &root,
        &readme,
        &[
            String::new(),
            "https://example.com".to_string(),
            "#anchor".to_string(),
            "%2Fguide.md".to_string(),
            "/docs/guide.md".to_string(),
            "./guide.md".to_string(),
            "../docs/guide.md".to_string(),
            "../../outside.md".to_string(),
            "missing.md".to_string(),
            "guide.md".to_string(),
        ],
        &known,
        &remapper,
    );
    assert!(
        links
            .iter()
            .any(|link| link.path == guide && link.whole_file),
        "{links:?}"
    );
}

#[test]
fn normalize_inside_rejects_paths_that_escape_the_root() {
    let root = PathBuf::from("/repo");
    assert_eq!(
        super::links::normalize_inside(&root, &root.join("docs").join("guide.md")),
        Some(root.join("docs").join("guide.md"))
    );
    assert!(super::links::normalize_inside(&root, &root.join("..").join("outside.md")).is_none());
}

#[test]
fn option_defaults_are_empty() {
    assert!(Options::default().groups.is_empty());
    assert!(Group::default().parents.is_empty());
}

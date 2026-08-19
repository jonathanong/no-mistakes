use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
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

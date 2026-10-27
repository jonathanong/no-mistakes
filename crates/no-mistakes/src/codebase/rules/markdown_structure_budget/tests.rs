use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use pulldown_cmark::{CodeBlockKind, Event, Options as MarkdownOptions, Parser, Tag};

// Output cases moved to cli_mrf_snapshot_markdown_structure_budget. Keep parser
// counts, line-ending fixture bytes, and errors that abort before check JSON.

fn line_count(content: &str) -> usize {
    super::super::markdown_facts::markdown_line_count(content)
}

fn counts(content: &str) -> (usize, usize) {
    let mut tables = 0;
    let mut mermaid = 0;
    for event in Parser::new_ext(content, MarkdownOptions::all()) {
        match event {
            Event::Start(Tag::Table(_)) => tables += 1,
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(info)))
                if info
                    .split_whitespace()
                    .next()
                    .is_some_and(|token| token.eq_ignore_ascii_case("mermaid")) =>
            {
                mermaid += 1
            }
            _ => {}
        }
    }
    (tables, mermaid)
}

fn config(options: &str, include: &[&str], exclude: &[&str]) -> NoMistakesConfig {
    NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            include: include.iter().map(|item| (*item).to_string()).collect(),
            exclude: exclude.iter().map(|item| (*item).to_string()).collect(),
            options: serde_yaml::from_str(options).unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    }
}

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/rules/markdown-structure-budget")
        .join(name)
}

fn run(
    root: &Path,
    config: &NoMistakesConfig,
    relative_files: &[&str],
) -> Result<Vec<RuleFinding>> {
    let files = relative_files
        .iter()
        .map(|file| root.join(file))
        .collect::<Vec<_>>();
    let sources = super::super::source_store_for_files(&files);
    let mut plan = super::super::markdown_facts::MarkdownFactPlan::default();
    plan.request_pulldown(super::super::markdown_scope::markdown_files(&files));
    let facts = super::super::markdown_facts::MarkdownFactMap::prepare(&plan, &sources);
    check_with_files_sources_and_facts(root, config, &files, &facts)
}

#[test]
fn counts_gfm_tables_and_mermaid_fences() {
    assert_eq!(
        counts("| a |\n| --- |\n| b |\n```mermaid\ngraph TD\n```"),
        (1, 1)
    );
    assert_eq!(counts("    ```mermaid\n"), (0, 0));
}

#[test]
fn line_count_treats_empty_markdown_as_zero_lines() {
    assert_eq!(line_count(""), 0);
}

#[test]
fn saved_line_ending_fixtures_keep_cr_and_crlf() {
    let root = fixture(".");
    let crlf = std::fs::read(root.join("exact-crlf.md")).unwrap();
    assert!(crlf.windows(2).any(|pair| pair == b"\r\n"));
    let cr = std::fs::read(root.join("exact-cr.md")).unwrap();
    assert!(cr.contains(&b'\r'));
    assert!(!cr.windows(2).any(|pair| pair == b"\r\n"));
}

#[test]
fn rejects_duplicate_baseline_keys_across_external_project_roots() {
    let root = fixture("external-request");
    let mut config = config("", &[], &[]);
    for name in ["external-one", "external-two"] {
        config.projects.insert(
            name.to_string(),
            crate::config::v2::schema::Project {
                root: Some(
                    root.parent()
                        .unwrap()
                        .join(name)
                        .to_string_lossy()
                        .to_string(),
                ),
                ..Default::default()
            },
        );
    }
    config.rules[0].scope = None;
    config.rules[0].projects = vec!["external-one".to_string(), "external-two".to_string()];
    let error = run(
        &root,
        &config,
        &["../external-one/CLAUDE.md", "../external-two/CLAUDE.md"],
    )
    .unwrap_err();
    assert!(error
        .to_string()
        .contains("ambiguous baseline key `CLAUDE.md`"));
}

#[test]
fn baseline_requires_tracked_valid_json() {
    let root = fixture("invalid-baseline");
    let config = config("baselineFile: baseline.json", &["**/*.md"], &[]);
    assert!(run(&root, &config, &["doc.md", "baseline.json"]).is_err());
    assert!(run(&root, &config, &["doc.md"])
        .unwrap_err()
        .to_string()
        .contains("tracked"));
}

#[test]
fn rejects_ambiguous_stale_external_baseline_keys() {
    let root = fixture("external-request");
    let mut config = config("baselineFile: baseline.json", &[], &[]);
    for name in ["external-one", "external-two"] {
        config.projects.insert(
            name.to_string(),
            crate::config::v2::schema::Project {
                root: Some(
                    root.parent()
                        .unwrap()
                        .join(name)
                        .to_string_lossy()
                        .to_string(),
                ),
                ..Default::default()
            },
        );
    }
    config.rules[0].scope = None;
    config.rules[0].projects = vec!["external-one".to_string(), "external-two".to_string()];
    let error = run(
        &root,
        &config,
        &[
            "baseline.json",
            "../external-one/one.md",
            "../external-two/two.md",
        ],
    )
    .unwrap_err();
    assert!(error
        .to_string()
        .contains("ambiguous baseline key `stale.md`"));
}

#[test]
fn reports_read_failures_instead_of_a_stale_deleted_baseline() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/rules/markdown-read-failure");
    let error = run(
        &root,
        &config(
            "maxLines: 1\nbaselineFile: baseline.json",
            &["**/*.md"],
            &[],
        ),
        &["unreadable.md", "baseline.json"],
    )
    .unwrap_err();
    assert!(error.to_string().contains(RULE_ID));
    assert!(error.to_string().contains("could not read Markdown file"));
}

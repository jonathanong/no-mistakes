use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use pulldown_cmark::{CodeBlockKind, Event, Options as MarkdownOptions, Parser, Tag};

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
fn full_check_uses_strict_thresholds_and_unicode_scalar_characters() {
    let root = fixture(".");
    let crlf = std::fs::read(root.join("exact-crlf.md")).unwrap();
    assert!(crlf.windows(2).any(|pair| pair == b"\r\n"));
    let cr = std::fs::read(root.join("exact-cr.md")).unwrap();
    assert!(cr.contains(&b'\r'));
    assert!(!cr.windows(2).any(|pair| pair == b"\r\n"));
    let options = "maxLines: 6\nmaxChars: 100\nmaxTables: 1\nmaxMermaid: 1";
    let findings = run(
        &root,
        &config(options, &["**/*.md"], &[]),
        &["exact-crlf.md", "over-budget.md"],
    )
    .unwrap();
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].file, "over-budget.md");
    let cr_only = run(
        &root,
        &config("maxLines: 5\nmaxTables: 0", &["**/*.md"], &[]),
        &["exact-cr.md"],
    )
    .unwrap();
    assert_eq!(cr_only.len(), 1, "{cr_only:#?}");
    assert_eq!(cr_only[0].file, "exact-cr.md");
    let unicode = run(
        &root,
        &config("maxLines: 99\nmaxChars: 3", &["**/*.md"], &[]),
        &["unicode.md"],
    )
    .unwrap();
    assert_eq!(unicode.len(), 1);
}

#[test]
fn counts_multiple_tables_and_case_insensitive_fenced_mermaid_only() {
    let root = fixture(".");
    let findings = run(
        &root,
        &config("maxLines: 1", &["**/*.md"], &[]),
        &["over-budget.md"],
    )
    .unwrap();
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("2 tables"));
    assert!(findings[0].message.contains("2 Mermaid"));
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
fn baseline_matches_exact_counts_and_stale_cases_fail() {
    let root = fixture("baseline-match");
    let options = "maxLines: 1\nbaselineFile: baseline.json";
    let files = ["doc.md", "baseline.json"];
    assert!(run(&root, &config(options, &["**/*.md"], &[]), &files)
        .unwrap()
        .is_empty());
    let excluded = run(&root, &config(options, &["**/*.md"], &["doc.md"]), &files).unwrap();
    assert_eq!(excluded.len(), 1);
    assert!(excluded[0].message.contains("deleted or excluded"));
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
fn immutable_baseline_variants_detect_resolved_and_changed_counts() {
    for (name, expected) in [
        ("baseline-resolved", "no longer"),
        ("baseline-changed", "no longer match"),
    ] {
        let root = fixture(name);
        let findings = run(
            &root,
            &config(
                "maxLines: 1\nbaselineFile: baseline.json",
                &["**/*.md"],
                &[],
            ),
            &["doc.md", "baseline.json"],
        )
        .unwrap();
        assert_eq!(findings.len(), 1, "{name}: {findings:#?}");
        assert!(findings[0].message.contains(expected));
    }
}

#[test]
fn stale_external_baseline_entries_use_request_relative_finding_paths() {
    let root = fixture("external-request");
    let external = root.parent().unwrap().join("external-project");
    let mut config = config("baselineFile: baseline.json", &[], &[]);
    config.projects.insert(
        "external".to_string(),
        crate::config::v2::schema::Project {
            root: Some(external.to_string_lossy().to_string()),
            ..Default::default()
        },
    );
    config.rules[0].scope = None;
    config.rules[0].projects = vec!["external".to_string()];
    let findings = run(
        &root,
        &config,
        &["baseline.json", "../external-project/CLAUDE.md"],
    )
    .unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "../external-project/stale.md");
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
fn ignores_tracked_paths_without_a_source_snapshot() {
    let root = fixture(".");
    let findings = run(
        &root,
        &config("maxLines: 1", &["**/*.md"], &[]),
        &["over-budget.md", "missing.md"],
    )
    .unwrap();
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].file, "over-budget.md");
}

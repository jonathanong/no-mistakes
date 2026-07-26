use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};

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
    check_with_files_and_sources(root, config, &files, &sources)
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
fn full_check_uses_strict_thresholds_and_unicode_scalar_characters() {
    let root = fixture(".");
    let crlf = std::fs::read(root.join("exact-crlf.md")).unwrap();
    assert!(crlf.windows(2).any(|pair| pair == b"\r\n"));
    let options = "maxLines: 6\nmaxChars: 100\nmaxTables: 1\nmaxMermaid: 1";
    let findings = run(
        &root,
        &config(options, &["**/*.md"], &[]),
        &["exact-crlf.md", "over-budget.md"],
    )
    .unwrap();
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].file, "over-budget.md");
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

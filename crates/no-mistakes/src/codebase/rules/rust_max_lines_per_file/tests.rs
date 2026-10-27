use super::tests_support::*;
use super::*;
use crate::codebase::rules::{path_filter, sort_findings, target_roots};
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::{Path, PathBuf};

// Output cases moved to cli_mrf_snapshot_rust_max_lines. Keep physical line
// count, unreadable paths, absolute roots, and finding order below.

fn config_with_rule(yaml: &str) -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str(yaml).unwrap(),
        ..Default::default()
    });
    config
}

fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
) -> anyhow::Result<Vec<RuleFinding>> {
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let opts = rule.try_rule_options().unwrap();
        let roots = normalize_roots(&opts, root, &target_roots(root, config, rule));
        let files: Vec<PathBuf> = all_files
            .iter()
            .filter(|path| {
                roots.iter().any(|rule_root| {
                    crate::codebase::ts_resolver::normalize_path(path).starts_with(rule_root)
                }) && !is_excluded(root, path, &opts.excludes)
            })
            .cloned()
            .collect();
        let files = path_filter::filter_rule_files(root, config, rule, &files)?;
        findings.extend(scan(root, &opts, &files)?);
    }
    sort_findings(&mut findings);
    Ok(findings)
}

fn fixture(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/rust-max-lines-per-file/fixture")
        .join(path)
}

fn read_fixture(path: &str) -> String {
    std::fs::read_to_string(fixture(path)).unwrap()
}

#[test]
fn check_sorts_multiple_findings_by_file() {
    let root = fixture("sort-by-file");
    let config = config_with_rule("{srcMax: 1}");
    let findings = check(&root, &config).unwrap();
    // Snapshot normalization sorts arrays, so it does not lock emission order.
    assert_eq!(findings.len(), 2);
    assert_eq!(findings[0].file, "a.rs");
    assert_eq!(findings[1].file, "b.rs");
}

#[test]
fn check_file_reports_unreadable_file() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("missing.rs");
    let finding = check_file(&path, tmp.path(), 5).unwrap();
    assert_eq!(finding.target.as_deref(), Some("unanalyzable"));
}

#[test]
fn check_with_files_respects_roots() {
    let root = fixture("roots");
    let sub = root.join("sub");
    let outside = root.join("big.rs");
    let inside = sub.join("big.rs");
    let sub_str = sub.to_str().unwrap();
    let config = config_with_rule(&format!("{{srcMax: 3, roots: [\"{sub_str}\"]}}"));
    let all_files = vec![outside, inside];
    let findings = check_with_files(&root, &config, &all_files).unwrap();
    assert_eq!(
        findings.len(),
        1,
        "only the file within roots should be flagged"
    );
    assert!(findings[0].file.contains("sub"));
}

#[test]
fn physical_lines_can_exceed_the_code_line_limit() {
    let source = read_fixture("unit/physical_vs_code.rs");
    // Check JSON has no physical line count. srcMax 2 and 3 snapshots lock code lines.
    assert_eq!(source.lines().count(), 9);
}

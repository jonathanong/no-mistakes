use super::tests_support::*;
use super::*;
use crate::codebase::rules::{path_filter, rust_max_lines_per_file, sort_findings, target_roots};
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::{Path, PathBuf};

// Output cases moved to cli_mrf_snapshot_rust_no_inline_tests. Keep absolute
// roots and finding order below.

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
                    && !rust_max_lines_per_file::is_test_file(root, path)
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
        .join("../../test-cases/rules/rust-no-inline-tests/fixture")
        .join(path)
}

#[test]
fn check_with_files_respects_roots() {
    let root = fixture("roots");
    let sub = root.join("sub");
    let outside = root.join("a.rs");
    let inside = sub.join("b.rs");
    let sub_str = sub.to_str().unwrap();
    let config = config_with_rule(&format!("{{roots: [\"{sub_str}\"]}}"));
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
fn check_sorts_by_file_then_line() {
    let root = fixture("sort-by-file");
    let config = config_with_rule("{}");
    let findings = check(&root, &config).unwrap();
    // Snapshot normalization sorts arrays, so it does not lock emission order.
    assert_eq!(findings.len(), 3);
    for i in 1..findings.len() {
        let a = (&findings[i - 1].file, findings[i - 1].line);
        let b = (&findings[i].file, findings[i].line);
        assert!(a <= b);
    }
}

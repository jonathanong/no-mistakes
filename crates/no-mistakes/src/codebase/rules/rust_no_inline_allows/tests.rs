use super::tests_support::*;
use super::*;
use crate::codebase::rules::{path_filter, rust_max_lines_per_file, sort_findings, target_roots};
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::{Path, PathBuf};

// Output cases moved to cli_mrf_snapshot_rust_no_inline_allows. Keep unreadable
// paths, absolute roots, and unconfigured-rule emptiness below.

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
        findings.extend(scan(root, &files)?);
    }
    sort_findings(&mut findings);
    Ok(findings)
}

fn fixture(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/rust-no-inline-allows/fixture")
        .join(path)
}

#[test]
fn unreadable_file_reports_unanalyzable_input() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("missing.rs");
    assert_eq!(
        check_file(&path, tmp.path())[0].target.as_deref(),
        Some("unanalyzable")
    );
}

#[test]
fn check_with_files_respects_absolute_roots() {
    let root = fixture("roots");
    let sub = root.join("sub");
    let outside = root.join("a.rs");
    let inside = sub.join("b.rs");
    let config = config_with_rule(&format!("{{roots: [\"{}\"]}}", sub.display()));
    let findings = check_with_files(&root, &config, &[outside, inside]).unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "sub/b.rs");
}

#[test]
fn helpers_cover_unrooted_excludes_absolute_roots_and_missing_files() {
    let root = Path::new("/repo");
    assert!(is_excluded(
        root,
        Path::new("/elsewhere/generated.rs"),
        &["generated".to_string()]
    ));
    assert!(!is_excluded(
        root,
        Path::new("/repo/src/lib.rs"),
        &["generated".to_string()]
    ));
    let opts = Options {
        roots: Some(vec![PathBuf::from("src"), PathBuf::from("/abs")]),
        excludes: vec![],
    };
    let roots = normalize_roots(&opts, root, &[PathBuf::from("/repo/fallback")]);
    assert!(roots.contains(&PathBuf::from("/repo/src")));
    assert!(roots.contains(&PathBuf::from("/abs")));
    assert_eq!(
        check_file(Path::new("/repo/missing.rs"), root)[0]
            .target
            .as_deref(),
        Some("unanalyzable")
    );
}

#[test]
fn check_discovers_absolute_roots_and_skips_unconfigured_rules() {
    let root = fixture("roots");
    let sub = root.join("sub");
    let absolute = config_with_rule(&format!("{{roots: [\"{}\"]}}", sub.display()));
    let findings = check(&root, &absolute).unwrap();
    assert_eq!(findings.len(), 1);
    assert!(
        findings[0].file.ends_with("b.rs"),
        "unexpected file: {}",
        findings[0].file
    );

    assert!(check(&root, &NoMistakesConfig::default())
        .unwrap()
        .is_empty());
}

use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};

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

fn fixture(path: &str) -> PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/rules/require-test-per-subdir")
        .join(path)
}

#[test]
fn pass_fixture_no_findings() {
    let root = fixture("pass");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let files: Vec<PathBuf> = [
        root.join("agents/email/index.mts"),
        root.join("agents/email/index.test.mts"),
        root.join("agents/_shared/helpers.mts"),
    ]
    .into();
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert!(
        findings.is_empty(),
        "expected no findings, got: {findings:?}"
    );
}

#[test]
fn fail_fixture_reports_missing_test() {
    let root = fixture("fail");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let files: Vec<PathBuf> = [root.join("agents/email/index.mts")].into();
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert_eq!(findings.len(), 1, "expected one finding, got: {findings:?}");
    assert!(
        findings[0].message.contains("agents/email"),
        "{}",
        findings[0].message
    );
    assert!(
        findings[0].message.contains("*.test.mts"),
        "{}",
        findings[0].message
    );
}

#[test]
fn no_op_when_roots_empty() {
    let tmp = tempfile::tempdir().unwrap();
    let config = config_with_rule("{}");
    let findings = check(tmp.path(), &config).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn excludes_dirs_are_skipped() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let shared = root.join("agents/_shared/helpers.mts");
    std::fs::create_dir_all(shared.parent().unwrap()).unwrap();
    std::fs::write(&shared, "").unwrap();
    let config =
        config_with_rule("roots: [agents]\nexcludeDirs: [_shared]\ntestGlob: \"*.test.mts\"");
    let files = vec![shared];
    let findings = check_with_files(root, &config, &files).unwrap();
    assert!(
        findings.is_empty(),
        "_shared should be excluded, got: {findings:?}"
    );
}

#[test]
fn reports_subdir_without_matching_test() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let src = root.join("agents/email/index.mts");
    std::fs::create_dir_all(src.parent().unwrap()).unwrap();
    std::fs::write(&src, "").unwrap();
    let config = config_with_rule("roots: [agents]\ntestGlob: \"*.test.mts\"");
    let files = vec![src];
    let findings = check_with_files(root, &config, &files).unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].line, 1);
    assert!(findings[0].message.contains("agents/email"));
}

#[test]
fn no_finding_when_test_file_present() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let src = root.join("agents/email/index.mts");
    let test = root.join("agents/email/index.test.mts");
    std::fs::create_dir_all(src.parent().unwrap()).unwrap();
    std::fs::write(&src, "").unwrap();
    std::fs::write(&test, "").unwrap();
    let config = config_with_rule("roots: [agents]\ntestGlob: \"*.test.mts\"");
    let files = vec![src, test];
    let findings = check_with_files(root, &config, &files).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn root_level_files_not_treated_as_subdirs() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let f = root.join("agents/root-level.mts");
    std::fs::create_dir_all(f.parent().unwrap()).unwrap();
    std::fs::write(&f, "").unwrap();
    // root-level.mts is directly under agents/, so agents/ itself is the only
    // "root" — no first-level subdirectory exists, so no finding.
    let config = config_with_rule("roots: [agents]\ntestGlob: \"*.test.mts\"");
    let files = vec![f];
    let findings = check_with_files(root, &config, &files).unwrap();
    // There are no first-level subdirs (files are direct children of agents/),
    // so no findings should be emitted.
    assert!(
        findings.is_empty(),
        "root-level file should not create a subdir finding, got: {findings:?}"
    );
}

#[test]
fn default_test_glob_matches_any_extension() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let src = root.join("agents/email/index.ts");
    let test = root.join("agents/email/index.test.ts");
    std::fs::create_dir_all(src.parent().unwrap()).unwrap();
    std::fs::write(&src, "").unwrap();
    std::fs::write(&test, "").unwrap();
    let config = config_with_rule("roots: [agents]");
    let files = vec![src, test];
    let findings = check_with_files(root, &config, &files).unwrap();
    assert!(
        findings.is_empty(),
        "default glob *.test.* should match .test.ts"
    );
}

#[test]
fn check_with_files_empty_roots_is_noop() {
    // Exercises the opts.roots.is_empty() branch (line 46) in check_with_files.
    let tmp = tempfile::tempdir().unwrap();
    let config = config_with_rule("{}");
    let findings = check_with_files(tmp.path(), &config, &[]).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn check_standalone_with_non_empty_roots_discovers_files() {
    // Exercises check(): non-empty roots → target_roots + discover_files.
    // In a non-git temp directory, discover_files returns empty, so no findings.
    let tmp = tempfile::tempdir().unwrap();
    let config = config_with_rule("roots: [agents]\ntestGlob: \"*.test.mts\"");
    let findings = check(tmp.path(), &config).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn files_outside_root_are_skipped_in_first_level_subdirs() {
    // Exercises strip_prefix error branch (line 101): file outside root is skipped.
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let other_dir = tempfile::tempdir().unwrap();
    let outside = other_dir.path().join("unrelated/file.ts");
    std::fs::create_dir_all(outside.parent().unwrap()).unwrap();
    std::fs::write(&outside, "").unwrap();
    // No in-scope files → no subdirs → no findings
    let config = config_with_rule("roots: [agents]\ntestGlob: \"*.test.mts\"");
    let files = vec![outside];
    let findings = check_with_files(root, &config, &files).unwrap();
    assert!(findings.is_empty(), "outside files don't generate subdirs");
}

#[test]
fn files_in_root_but_outside_spec_subroot_skip_strip_prefix() {
    // Exercises line 115: strip_prefix(abs_root) fails for files that pass the
    // target_roots filter (under root) but are not under the spec's sub-root (agents/).
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    // File is under root but NOT under root/agents — strip_prefix(agents/) fails.
    let outside_agents = root.join("other-dir/file.mts");
    std::fs::create_dir_all(outside_agents.parent().unwrap()).unwrap();
    std::fs::write(&outside_agents, "").unwrap();
    let config = config_with_rule("roots: [agents]\ntestGlob: \"*.test.mts\"");
    let files = vec![outside_agents];
    let findings = check_with_files(root, &config, &files).unwrap();
    assert!(
        findings.is_empty(),
        "files outside the spec subroot should not generate subdirs or findings"
    );
}

#[test]
fn absolute_root_path_is_used_directly() {
    // Exercises rule_root.is_absolute() branch (line 62) in scan().
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let abs_agents = root.join("agents");
    let src = abs_agents.join("email/index.mts");
    let test = abs_agents.join("email/index.test.mts");
    std::fs::create_dir_all(src.parent().unwrap()).unwrap();
    std::fs::write(&src, "").unwrap();
    std::fs::write(&test, "").unwrap();
    let yaml = format!(
        "roots:\n  - {}\ntestGlob: \"*.test.mts\"",
        abs_agents.display()
    );
    let config = config_with_rule(&yaml);
    let files = vec![src, test];
    let findings = check_with_files(root, &config, &files).unwrap();
    assert!(
        findings.is_empty(),
        "absolute root with test file should produce no findings: {findings:?}"
    );
}

#[test]
fn root_dir_itself_in_file_list_is_skipped_by_first_level_subdirs() {
    // Exercises line 105: when rel is empty (file == abs_root), components.next()
    // returns None and the file is skipped.
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let agents = root.join("agents");
    std::fs::create_dir_all(&agents).unwrap();
    // Pass the root dir itself in the file list — strip_prefix returns "" (empty path)
    let config = config_with_rule("roots: [agents]\ntestGlob: \"*.test.mts\"");
    let files = vec![agents.clone()];
    let findings = check_with_files(root, &config, &files).unwrap();
    assert!(
        findings.is_empty(),
        "root dir itself should be skipped, no subdirs found"
    );
}

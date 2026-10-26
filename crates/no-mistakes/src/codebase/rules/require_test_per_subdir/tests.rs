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
    crate::codebase::ts_resolver::normalize_path(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/require-test-per-subdir/fixture")
            .join(path),
    )
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
fn project_scoped_fixture_ignores_files_outside_target_roots() {
    let root = fixture("project-scoped-pass");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let findings = check(&root, &config).unwrap();
    assert!(
        findings.is_empty(),
        "project-scoped rule should ignore outside files, got: {findings:?}"
    );
}

#[test]
fn nested_test_dir_fixture_has_no_findings() {
    let root = fixture("nested-pass");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let findings = check(&root, &config).unwrap();
    assert!(
        findings.is_empty(),
        "nested test directory should satisfy the subdir requirement: {findings:?}"
    );
}

#[test]
fn direct_child_fixture_rejects_nested_descendant_test() {
    let root = fixture("direct-child-fail");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let files: Vec<PathBuf> = [
        root.join("agents/email/index.mts"),
        root.join("agents/email/nested/index.test.mts"),
    ]
    .into();
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert_eq!(findings.len(), 1, "expected one finding, got: {findings:?}");
    assert!(
        findings[0].message.contains("agents/email"),
        "{}",
        findings[0].message
    );
}

#[test]
fn direct_child_fixture_accepts_direct_test() {
    let root = fixture("direct-child-pass");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let files: Vec<PathBuf> = [
        root.join("agents/email/index.mts"),
        root.join("agents/email/index.test.mts"),
        root.join("agents/email/nested/extra.test.mts"),
    ]
    .into();
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert!(
        findings.is_empty(),
        "direct child test should satisfy directChild mode: {findings:?}"
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
    let root = fixture("exclude-shared");
    let shared = root.join("agents/_shared/helpers.mts");
    let config =
        config_with_rule("roots: [agents]\nexcludeDirs: [_shared]\ntestGlob: \"*.test.mts\"");
    let files = vec![shared];
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert!(
        findings.is_empty(),
        "_shared should be excluded, got: {findings:?}"
    );
}

#[test]
fn reports_subdir_without_matching_test() {
    let root = fixture("email-source-only");
    let src = root.join("agents/email/index.mts");
    let config = config_with_rule("roots: [agents]\ntestGlob: \"*.test.mts\"");
    let files = vec![src];
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].line, 1);
    assert!(findings[0].message.contains("agents/email"));
}

#[test]
fn no_finding_when_test_file_present() {
    let root = fixture("email-with-test");
    let src = root.join("agents/email/index.mts");
    let test = root.join("agents/email/index.test.mts");
    let config = config_with_rule("roots: [agents]\ntestGlob: \"*.test.mts\"");
    let files = vec![src, test];
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn root_level_files_not_treated_as_subdirs() {
    let root = fixture("root-level-file");
    let f = root.join("agents/root-level.mts");
    // root-level.mts is directly under agents/, so agents/ itself is the only
    // "root" — no first-level subdirectory exists, so no finding.
    let config = config_with_rule("roots: [agents]\ntestGlob: \"*.test.mts\"");
    let files = vec![f];
    let findings = check_with_files(&root, &config, &files).unwrap();
    // There are no first-level subdirs (files are direct children of agents/),
    // so no findings should be emitted.
    assert!(
        findings.is_empty(),
        "root-level file should not create a subdir finding, got: {findings:?}"
    );
}

#[test]
fn default_test_glob_matches_any_extension() {
    let root = fixture("default-glob");
    let src = root.join("agents/email/index.ts");
    let test = root.join("agents/email/index.test.ts");
    let config = config_with_rule("roots: [agents]");
    let files = vec![src, test];
    let findings = check_with_files(&root, &config, &files).unwrap();
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
    let bundle = fixture("outside-file");
    let root = bundle.join("root");
    let outside = bundle.join("outside/unrelated/file.ts");
    // No in-scope files → no subdirs → no findings
    let config = config_with_rule("roots: [agents]\ntestGlob: \"*.test.mts\"");
    let files = vec![outside];
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert!(findings.is_empty(), "outside files don't generate subdirs");
}

#[test]
fn files_in_root_but_outside_spec_subroot_skip_strip_prefix() {
    // Exercises line 115: strip_prefix(abs_root) fails for files that pass the
    // target_roots filter (under root) but are not under the spec's sub-root (agents/).
    let root = fixture("outside-spec-subroot");
    // File is under root but NOT under root/agents — strip_prefix(agents/) fails.
    let outside_agents = root.join("other-dir/file.mts");
    let config = config_with_rule("roots: [agents]\ntestGlob: \"*.test.mts\"");
    let files = vec![outside_agents];
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert!(
        findings.is_empty(),
        "files outside the spec subroot should not generate subdirs or findings"
    );
}

#[test]
fn absolute_root_path_is_used_directly() {
    // Exercises rule_root.is_absolute() branch (line 62) in scan().
    let root = fixture("email-with-test");
    let abs_agents = root.join("agents");
    let src = abs_agents.join("email/index.mts");
    let test = abs_agents.join("email/index.test.mts");
    let yaml = format!(
        "roots:\n  - {}\ntestGlob: \"*.test.mts\"",
        abs_agents.display()
    );
    let config = config_with_rule(&yaml);
    let files = vec![src, test];
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert!(
        findings.is_empty(),
        "absolute root with test file should produce no findings: {findings:?}"
    );
}

#[test]
fn root_dir_itself_in_file_list_is_skipped_by_first_level_subdirs() {
    // Exercises line 105: when rel is empty (file == abs_root), components.next()
    // returns None and the file is skipped.
    let root = fixture("root-dir-itself");
    let agents = root.join("agents");
    // Pass the root dir itself in the file list — strip_prefix returns "" (empty path)
    let config = config_with_rule("roots: [agents]\ntestGlob: \"*.test.mts\"");
    let files = vec![agents.clone()];
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert!(
        findings.is_empty(),
        "root dir itself should be skipped, no subdirs found"
    );
}

#[test]
fn parent_segment_root_reports_missing_test() {
    // `services/../agents` must resolve to `agents` before strip_prefix.
    let root = fixture("parent-segment");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let files = vec![root.join("agents/email/index.mts")];
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("agents/email"), "{findings:?}");
    assert!(!findings[0].message.contains("escapes"));
}

#[test]
fn dot_prefix_root_still_matches() {
    let root = fixture("parent-segment");
    let config = config_with_rule("roots: [\"./agents\"]\ntestGlob: \"*.test.mts\"");
    let file = root.join("agents/email/index.mts");
    let findings = check_with_files(&root, &config, &[file]).unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("agents/email"), "{findings:?}");
    assert!(!findings[0].message.contains("escapes"));
}

#[test]
fn escaping_root_is_one_finding_and_skips_outside_files() {
    let root = fixture("escape");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let inside = root.join("agents/email/index.mts");
    let outside = fixture("fail").join("agents/email/index.mts");
    let opts: Options = config.rule_applications(RULE_ID)[0]
        .try_rule_options()
        .unwrap();
    let findings = super::scan(&root, &config, &opts, &[inside.clone(), outside.clone()]).unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].file, ".no-mistakes.yml");
    assert!(
        findings[0].message.contains("escapes the repository"),
        "{findings:?}"
    );
    let outside_text = outside.display().to_string();
    assert!(
        findings.iter().all(|finding| {
            !finding.message.contains("no test file")
                && !finding.file.contains("index.mts")
                && !finding.message.contains(&outside_text)
                && !finding.file.contains(&outside_text)
        }),
        "{findings:?}"
    );
    let checked = check_with_files(&root, &config, &[inside]).unwrap();
    assert_eq!(checked.len(), 1, "{checked:?}");
    assert!(checked[0].message.contains("escapes the repository"));
    assert_eq!(checked[0].file, ".no-mistakes.yml");
}

#[test]
fn escaping_root_ignores_a_shorter_markdown_name() {
    let root = fixture("escape-markdown");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let files = vec![
        root.join(".no-mistakes.md"),
        root.join(".no-mistakes.yml"),
        root.join("agents/email/index.mts"),
    ];
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].file, ".no-mistakes.yml");
}

#[test]
fn escaping_root_stays_on_a_custom_config_excluded_from_the_scan() {
    let root = fixture("escape-custom");
    let config_path = root.join("tools/check.yml");
    let config = crate::config::v2::load_v2_config(&root, Some(&config_path)).unwrap();
    let files = vec![
        config_path,
        root.join(".no-mistakes.md"),
        root.join("agents/email/index.mts"),
    ];
    let mut findings = check_with_files(&root, &config, &files).unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].file, "tools/check.yml");
    let sources = super::super::source_store_for_files(&files);
    super::super::suppress_rule_findings_with_sources(&root, &mut findings, &sources);
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn escaping_root_is_suppressed_from_the_config_file() {
    let root = fixture("escape-disabled");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let files = vec![
        root.join(".no-mistakes.yml"),
        root.join("agents/email/index.mts"),
    ];
    let mut findings = check_with_files(&root, &config, &files).unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    let sources = super::super::source_store_for_files(&files);
    super::super::suppress_rule_findings_with_sources(&root, &mut findings, &sources);
    assert!(findings.is_empty(), "{findings:?}");
}

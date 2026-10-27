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

// Output cases moved to cli_rra_snapshot_require_test_per_subdir.

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
fn no_op_when_roots_empty() {
    let tmp = tempfile::tempdir().unwrap();
    let config = config_with_rule("{}");
    let findings = check(tmp.path(), &config).unwrap();
    assert!(findings.is_empty());
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

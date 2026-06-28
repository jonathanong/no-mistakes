use super::*;
use crate::config::v2::load_v2_config;
use std::path::{Path, PathBuf};

fn fixture_root(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/vitest-ci-path-coverage")
            .join(name),
    )
}

fn files(root: &Path) -> Vec<PathBuf> {
    crate::codebase::ts_source::discover_files(root, &[])
}

fn rule_options_mut(
    config: &mut crate::config::v2::schema::NoMistakesConfig,
) -> &mut serde_yaml::Value {
    &mut config
        .rules
        .iter_mut()
        .find(|rule| rule.rule == RULE_ID)
        .expect("fixture should include vitest-ci-path-coverage rule")
        .options
}

#[test]
fn reports_source_input_missed_by_too_narrow_ci_filter() {
    let root = fixture_root("fixture");
    let config = load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    let findings = check_with_files(&root, &config, &files(&root)).unwrap();

    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].rule, RULE_ID);
    assert_eq!(findings[0].file, ".github/workflows/ci.yml");
    assert!(findings[0].message.contains("ts-shared/utils/index.mts"));
    assert!(
        findings[0]
            .message
            .contains("Vitest project `ts-shared` full-suite trigger path"),
        "{}",
        findings[0].message
    );
}

#[test]
fn passes_when_ci_filter_covers_source_inputs() {
    let root = fixture_root("pass");
    let config = load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    let findings = check_with_files(&root, &config, &files(&root)).unwrap();

    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn reports_missing_filter_mapping_for_matched_project_files() {
    let root = fixture_root("fixture");
    let mut config = load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    *rule_options_mut(&mut config) = serde_yaml::from_str(
        r#"
includeFullSuiteTriggers: false
projectFilters:
  backend: [missing]
workflows:
  - path: .github/workflows/ci.yml
    job: detect-changes
    stepId: filter
"#,
    )
    .unwrap();

    let findings = check_with_files(&root, &config, &files(&root)).unwrap();

    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert!(
        findings[0]
            .message
            .contains("Vitest project `backend` test include paths are not mapped"),
        "{}",
        findings[0].message
    );
}

#[test]
fn source_globs_by_project_are_checked() {
    let root = fixture_root("fixture");
    let mut config = load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    *rule_options_mut(&mut config) = serde_yaml::from_str(
        r#"
includeVitestProjectGlobs: false
includeFullSuiteTriggers: false
sourceGlobsByProject:
  ts-shared: ["ts-shared/**/*.mts"]
projectFilters:
  ts-shared: [backend]
workflows:
  - path: .github/workflows/ci.yml
    job: detect-changes
    stepId: filter
"#,
    )
    .unwrap();

    let findings = check_with_files(&root, &config, &files(&root)).unwrap();

    assert_eq!(findings.len(), 2, "{findings:#?}");
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("configured source path")),
        "{findings:#?}"
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("glob witness path")),
        "{findings:#?}"
    );
}

#[test]
fn negated_filters_remove_coverage() {
    let root = fixture_root("negated");
    let config = load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();

    let findings = check_with_files(&root, &config, &files(&root)).unwrap();

    assert_eq!(findings.len(), 2, "{findings:#?}");
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("ts-shared/utils/index.mts")),
        "{findings:#?}"
    );
}

#[test]
fn default_paths_filter_quantifier_does_not_treat_negations_as_exclusions() {
    let root = fixture_root("negated-default");
    let config = load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();

    let findings = check_with_files(&root, &config, &files(&root)).unwrap();

    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn external_filter_files_are_loaded() {
    let root = fixture_root("external");
    let config = load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();

    let findings = check_with_files(&root, &config, &files(&root)).unwrap();

    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn change_type_filter_rules_preserve_paths() {
    let root = fixture_root("change-type");
    let config = load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();

    let findings = check_with_files(&root, &config, &files(&root)).unwrap();

    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn non_paths_filter_steps_are_ignored() {
    let root = fixture_root("unrelated-action");
    let config = load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();

    let findings = check_with_files(&root, &config, &files(&root)).unwrap();

    assert_eq!(findings.len(), 2, "{findings:#?}");
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("src/index.ts")),
        "{findings:#?}"
    );
}

#[test]
fn reports_glob_witness_missed_by_too_shallow_ci_filter() {
    let root = fixture_root("glob-witness");
    let config = load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();

    let findings = check_with_files(&root, &config, &files(&root)).unwrap();

    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert!(
        findings[0]
            .message
            .contains("src/__no_mistakes_witness__/nested/__no_mistakes_witness__.ts"),
        "{}",
        findings[0].message
    );
    assert!(
        findings[0].message.contains("glob witness path"),
        "{}",
        findings[0].message
    );
}

#[test]
fn invalid_filter_globs_are_reported_as_findings() {
    let root = fixture_root("malformed");
    let config = load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    let findings = check_with_files(&root, &config, &files(&root)).unwrap();

    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("contains invalid glob")),
        "{findings:#?}"
    );
}

#[test]
fn double_star_suffix_globs_match_repo_root_files() {
    let compiled = globs::compile_patterns(&["**.ts".to_string()]).unwrap();

    assert!(globs::selected_by(&compiled, "index.ts"));
    assert!(globs::selected_by(&compiled, "src/index.ts"));
}

#[test]
fn full_suite_trigger_negations_stay_at_glob_start() {
    let root = fixture_root("fixture");
    let mut config = load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    config.test_plan.vitest.full_suite_triggers.projects.insert(
        "ts-shared".to_string(),
        serde_yaml::from_str(
            r#"
- "**"
- "!utils/**"
"#,
        )
        .unwrap(),
    );

    let findings = check_with_files(&root, &config, &files(&root)).unwrap();

    assert!(findings.is_empty(), "{findings:#?}");
}

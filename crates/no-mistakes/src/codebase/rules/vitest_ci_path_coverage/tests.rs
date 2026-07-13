use super::*;

fn scan(
    root: &Path,
    config: &NoMistakesConfig,
    opts: &Options,
    files: &[PathBuf],
    all_files: &[PathBuf],
    snapshot: &crate::codebase::ts_source::VisiblePathSnapshot,
) -> Result<Vec<RuleFinding>> {
    scan_with_catalog(root, config, opts, files, all_files, snapshot, None)
}
use crate::codebase::rules::vitest_ci_path_coverage::projects::CoverageSource;
use crate::config::v2::load_v2_config;
use std::collections::BTreeMap;
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
fn scan_returns_no_findings_when_no_files_are_in_scope() {
    let root = fixture_root("fixture");
    let config = load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    let opts = Options::default();
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);

    let findings = scan(&root, &config, &opts, &[], &[], &snapshot).unwrap();

    assert!(findings.is_empty());
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
fn prepared_vitest_catalog_matches_standalone_coverage_loading() {
    let root = fixture_root("fixture");
    let config = load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    let files = files(&root);
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let visible = snapshot.paths_for(&root);
    let tsconfig =
        crate::codebase::ts_resolver::resolve_tsconfig_from_visible(None, &root, &visible).unwrap();
    let catalog =
        super::super::prepare_vitest_project_catalog(&root, &config, &snapshot, &tsconfig);

    let standalone = check_with_files(&root, &config, &files).unwrap();
    let prepared = check_with_files_from_snapshot_and_catalog(
        &root,
        &config,
        &files,
        &snapshot,
        Some(&catalog),
    )
    .unwrap();

    assert_eq!(prepared, standalone);
}

#[test]
fn full_suite_trigger_inputs_are_checked_even_when_rule_files_are_scoped() {
    let root = fixture_root("fixture");
    let config = load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    let mut project_filters = BTreeMap::new();
    project_filters.insert("ts-shared".to_string(), vec!["backend".to_string()]);
    let opts = Options {
        include_vitest_project_globs: Some(false),
        project_filters,
        workflows: vec![WorkflowSelector {
            path: ".github/workflows/ci.yml".to_string(),
            job: "detect-changes".to_string(),
            step_id: "filter".to_string(),
        }],
        ..Options::default()
    };
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);

    let findings = scan(&root, &config, &opts, &[], &files(&root), &snapshot).unwrap();

    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert!(
        findings[0].message.contains("ts-shared/utils/index.mts"),
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
fn mapped_filter_names_default_to_project_and_missing_mapping_points_at_project() {
    let opts = Options::default();
    assert_eq!(mapped_filter_names(&opts, "backend"), vec!["backend"]);

    let finding = missing_mapping_finding(
        ".github/workflows/ci.yml",
        &CoverageUnit {
            project: "backend".to_string(),
            source: CoverageSource::TestInclude,
            patterns: Vec::new(),
        },
    );

    assert_eq!(finding.file, ".github/workflows/ci.yml");
    assert_eq!(finding.target.as_deref(), Some("backend"));
    assert!(finding.message.contains("options.projectFilters.backend"));
}

#[test]
fn coverage_paths_include_real_files_and_recursive_witnesses_once() {
    let root = fixture_root("glob-witness");
    let unit = CoverageUnit {
        project: "backend".to_string(),
        source: CoverageSource::ConfiguredSource,
        patterns: vec!["src/**/*.ts".to_string(), "!src/generated/**".to_string()],
    };
    let files = vec![root.join("src/index.ts"), root.join("src/index.ts")];

    let paths = coverage_paths::coverage_paths(&root, &unit, &files).unwrap();

    assert_eq!(
        paths
            .iter()
            .map(|path| path.rel.as_str())
            .collect::<Vec<_>>(),
        vec![
            "src/index.ts",
            "src/__no_mistakes_witness__/nested/__no_mistakes_witness__.ts"
        ]
    );
    assert!(!paths[0].synthetic);
    assert!(paths[1].synthetic);
}

#[test]
fn coverage_paths_reports_invalid_glob_context() {
    let root = fixture_root("glob-witness");
    let unit = CoverageUnit {
        project: "backend".to_string(),
        source: CoverageSource::ConfiguredSource,
        patterns: vec!["[".to_string()],
    };
    let error = coverage_paths::coverage_paths(&root, &unit, &[]).unwrap_err();

    assert!(error
        .to_string()
        .contains("invalid glob in vitest-ci-path-coverage backend"));
}

#[test]
fn witness_paths_skip_negated_and_broad_patterns() {
    assert!(
        coverage_paths::witness_paths(&["**".to_string(), "!src/**/*.ts".to_string()]).is_empty()
    );
    assert_eq!(
        coverage_paths::witness_paths(&["src/**.ts".to_string()]),
        vec!["src/__no_mistakes_witness__/nested.ts"]
    );
}

#[test]
fn witness_path_handles_supported_glob_tokens() {
    assert_eq!(
        coverage_paths::witness_path("src/**/{app,lib}/file[ab]?.\\*"),
        "src/__no_mistakes_witness__/nested/app/fileax.*"
    );
    assert_eq!(
        coverage_paths::witness_path("src/**/{app}/file.ts"),
        "src/__no_mistakes_witness__/nested/app/file.ts"
    );
    assert_eq!(coverage_paths::witness_path("[ab].ts"), "a.ts");
}

#[test]
fn double_star_suffix_globs_match_repo_root_files() {
    let compiled = globs::compile_patterns(&["**.ts".to_string()]).unwrap();

    assert!(globs::selected_by(&compiled, "index.ts"));
    assert!(globs::selected_by(&compiled, "src/index.ts"));

    let nested = globs::compile_patterns(&["src/**.test.ts/fixtures/**".to_string()]).unwrap();
    assert!(globs::selected_by(
        &nested,
        "src/foo.test.ts/fixtures/data.ts"
    ));
}

#[test]
fn double_star_inside_path_segments_compiles() {
    let compiled = globs::compile_patterns(&["src/foo**bar.ts".to_string()]).unwrap();

    assert_eq!(compiled.len(), 1);
}

#[test]
fn full_suite_trigger_negations_do_not_exclude_broad_triggers() {
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

    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert!(findings[0].message.contains("ts-shared/utils/index.mts"));
}

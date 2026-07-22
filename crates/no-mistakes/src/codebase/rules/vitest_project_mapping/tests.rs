use super::*;

fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    check_with_files_and_catalog(root, config, all_files, None)
}
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use crate::integration_tests::project_config;
use std::path::Path;

fn fixture_root(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/vitest-project-mapping")
            .join(name),
    )
}

fn load_config(root: &Path) -> NoMistakesConfig {
    let mut config =
        crate::config::v2::load_v2_config(root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str("{}").unwrap(),
        ..Default::default()
    });
    config
}

#[test]
fn reports_unmapped_and_ambiguous_vitest_tests() {
    let root = fixture_root("fixture");
    let config = load_config(&root);
    let files = vec![
        root.join("src/a.test.ts"),
        root.join("src/shared.test.ts"),
        root.join("src/unmapped.test.ts"),
    ];
    let findings = check_with_files(&root, &config, &files).unwrap();

    assert_eq!(findings.len(), 2);
    assert_eq!(findings[0].file, "src/shared.test.ts");
    assert!(findings[0].message.contains("multiple Vitest projects"));
    assert_eq!(findings[1].file, "src/unmapped.test.ts");
    assert!(findings[1].message.contains("does not map"));
}

#[test]
fn prepared_vitest_catalog_matches_standalone_project_loading() {
    let root = fixture_root("fixture");
    let config = load_config(&root);
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let visible = snapshot.paths_for(&root);
    let tsconfig =
        crate::codebase::ts_resolver::resolve_tsconfig_from_visible(None, &root, &visible).unwrap();
    let tsconfig_catalog =
        crate::codebase::ts_resolver::TsConfigCatalog::forced(&root, tsconfig, None);
    let catalog =
        super::super::prepare_vitest_project_catalog(&root, &config, &snapshot, &tsconfig_catalog);

    let standalone = check_with_files(&root, &config, &files).unwrap();
    let prepared = check_with_files_and_catalog(&root, &config, &files, Some(&catalog)).unwrap();

    assert_eq!(prepared, standalone);
}

#[test]
fn default_extensions_include_spec_files() {
    let root = fixture_root("fixture");
    let config = load_config(&root);
    let files = vec![root.join("src/unmapped.spec.ts")];
    let findings = check_with_files(&root, &config, &files).unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "src/unmapped.spec.ts");
}

#[test]
fn default_extensions_include_javascript_test_files() {
    let root = fixture_root("fixture");
    let config = load_config(&root);
    let files = vec![root.join("src/unmapped.test.js")];
    let findings = check_with_files(&root, &config, &files).unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "src/unmapped.test.js");
}

#[test]
fn default_extensions_include_commonjs_typescript_test_files() {
    let root = fixture_root("fixture");
    let config = load_config(&root);
    let files = vec![root.join("src/unmapped.test.cts")];
    let findings = check_with_files(&root, &config, &files).unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "src/unmapped.test.cts");
}

#[test]
fn default_extensions_include_all_vitest_parser_variants() {
    let root = fixture_root("fixture");
    let config = load_config(&root);
    let files = vec![
        root.join("src/unmapped.test.mtsx"),
        root.join("src/__tests__/widget.mjsx"),
    ];
    let findings = check_with_files(&root, &config, &files).unwrap();

    assert_eq!(findings.len(), 2);
    assert_eq!(findings[0].file, "src/__tests__/widget.mjsx");
    assert_eq!(findings[1].file, "src/unmapped.test.mtsx");
}

#[test]
fn default_extensions_include_tests_directory_files() {
    let root = fixture_root("fixture");
    let config = load_config(&root);
    let files = vec![root.join("src/__tests__/routes.ts")];
    let findings = check_with_files(&root, &config, &files).unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "src/__tests__/routes.ts");
}

#[test]
fn scopes_can_limit_checked_test_candidates() {
    let root = fixture_root("fixture");
    let mut config = load_config(&root);
    config.rules[0].options =
        serde_yaml::from_str("scopes: [./ignored/../src/a.test.ts]\n").unwrap();
    let files = vec![
        root.join("src/a.test.ts"),
        root.join("src/shared.test.ts"),
        root.join("src/unmapped.test.ts"),
    ];
    let findings = check_with_files(&root, &config, &files).unwrap();

    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn root_scope_matches_all_relative_test_paths() {
    let root = fixture_root("fixture");
    let mut config = load_config(&root);
    config.rules[0].options = serde_yaml::from_str("scopes: [/]\n").unwrap();
    let files = vec![root.join("src/unmapped.test.ts")];
    let findings = check_with_files(&root, &config, &files).unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "src/unmapped.test.ts");
}

#[test]
fn configured_projects_and_custom_extensions_are_checked() {
    let root = fixture_root("fixture");
    let mut config = load_config(&root);
    config.tests.vitest.configs = None;
    config.tests.vitest.projects.insert(
        "custom".to_string(),
        serde_yaml::from_str(
            r#"
include: [src/custom.spec.ts]
"#,
        )
        .unwrap(),
    );
    config.rules[0].options =
        serde_yaml::from_str("testExtensions: [.spec.ts]\nscopes: [src]\n").unwrap();
    let files = vec![
        root.join("src/custom.spec.ts"),
        root.join("src/unmapped.spec.ts"),
    ];
    let findings = check_with_files(&root, &config, &files).unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "src/unmapped.spec.ts");
}

#[test]
fn configured_project_globs_normalize_relative_segments() {
    let root = fixture_root("fixture");
    let mut config = load_config(&root);
    config.tests.vitest.configs = None;
    config.tests.vitest.projects.insert(
        "custom".to_string(),
        serde_yaml::from_str(
            r#"
include: [./src/**/*.spec.ts]
exclude: [./src/unmapped.spec.ts]
"#,
        )
        .unwrap(),
    );
    config.rules[0].options =
        serde_yaml::from_str("testExtensions: [.spec.ts]\nscopes: [src]\n").unwrap();
    let files = vec![
        root.join("src/custom.spec.ts"),
        root.join("src/unmapped.spec.ts"),
    ];
    let findings = check_with_files(&root, &config, &files).unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "src/unmapped.spec.ts");
    assert!(findings[0].message.contains("does not map"));
}

#[test]
fn configured_project_globs_escape_literal_config_roots() {
    let root = fixture_root("fixture");
    let include = project_config::prefix_globs(
        &root,
        &root.join("packages/[tenant]"),
        &["src/**/*.test.ts".to_string()],
    );
    let projects = vec![projects::ProjectGlob {
        name: "tenant".to_string(),
        explicit: false,
        scope: Some("packages/[tenant]".to_string()),
        include: project_config::build_globset(&include).unwrap(),
        exclude: project_config::build_globset(&[]).unwrap(),
    }];

    assert_eq!(
        matching_projects("packages/[tenant]/src/a.test.ts", &projects),
        vec!["tenant".to_string()]
    );
}

#[test]
fn project_scoped_rules_match_scopes_against_project_relative_paths() {
    let root = fixture_root("fixture");
    let mut config = load_config(&root);
    config.projects.insert(
        "app".to_string(),
        crate::config::v2::schema::Project {
            root: Some("packages/app".to_string()),
            ..Default::default()
        },
    );
    config.rules[0].projects = vec!["app".to_string()];
    config.rules[0].options = serde_yaml::from_str("scopes: [src]\n").unwrap();
    config.tests.vitest.configs = None;
    config.tests.vitest.projects.insert(
        "unit".to_string(),
        serde_yaml::from_str("include: [packages/app/src/a.test.ts]\n").unwrap(),
    );
    let files = vec![root.join("packages/app/src/unmapped.test.ts")];
    let findings = check_with_files(&root, &config, &files).unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "packages/app/src/unmapped.test.ts");
}

#[test]
fn matching_paths_include_repo_relative_fallback_once() {
    let root = fixture_root("fixture");
    let file = root.join("src/a.test.ts");

    assert_eq!(
        relative_paths_for_matching(&root, &file, std::slice::from_ref(&root)),
        vec!["src/a.test.ts".to_string()]
    );
    assert_eq!(
        relative_paths_for_matching(&root, &file, &[]),
        vec!["src/a.test.ts".to_string()]
    );
}

#[test]
fn configured_projects_extend_auto_discovered_vitest_configs() {
    let root = fixture_root("fixture");
    let mut config = load_config(&root);
    config.tests.vitest.configs = None;
    config.tests.vitest.projects.insert(
        "custom".to_string(),
        serde_yaml::from_str(
            r#"
include: [src/custom.spec.ts]
"#,
        )
        .unwrap(),
    );
    config.rules[0].options = serde_yaml::from_str("scopes: [src]\n").unwrap();
    let files = vec![root.join("src/a.test.ts"), root.join("src/custom.spec.ts")];
    let findings = check_with_files(&root, &config, &files).unwrap();

    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn configured_projects_extend_explicit_vitest_config_projects() {
    let root = fixture_root("fixture");
    let mut config = load_config(&root);
    config.tests.vitest.configs = Some(crate::config::v2::schema::StringOrList::One(
        "vitest.config.mts".to_string(),
    ));
    config.tests.vitest.projects.insert(
        "custom".to_string(),
        serde_yaml::from_str("include: [src/custom.spec.ts]\n").unwrap(),
    );
    config.rules[0].options = serde_yaml::from_str("scopes: [src]\n").unwrap();
    let files = vec![root.join("src/a.test.ts"), root.join("src/custom.spec.ts")];
    let findings = check_with_files(&root, &config, &files).unwrap();

    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn configured_project_replaces_discovered_project_with_same_name() {
    let root = fixture_root("fixture");
    let mut config = load_config(&root);
    config.tests.vitest.configs = None;
    config.tests.vitest.projects.insert(
        "unit-a".to_string(),
        serde_yaml::from_str(
            r#"
include: [src/a.test.ts]
"#,
        )
        .unwrap(),
    );
    let files = vec![root.join("src/a.test.ts")];
    let findings = check_with_files(&root, &config, &files).unwrap();

    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn explicit_projects_only_handles_dynamic_vitest_configs() {
    let root = fixture_root("dynamic-config");
    let mut config = load_config(&root);
    config.rules[0].options = serde_yaml::from_str("explicitProjectsOnly: true\n").unwrap();
    let files = vec![
        root.join("backend/api.test.ts"),
        root.join("tooling/lint.test.ts"),
        root.join("web/page.test.ts"),
        root.join("web/skipped.generated.test.ts"),
        root.join("orphan.test.ts"),
    ];
    let findings = check_with_files(&root, &config, &files).unwrap();

    let finding_files = findings
        .iter()
        .map(|finding| finding.file.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        finding_files,
        vec!["orphan.test.ts", "web/skipped.generated.test.ts"]
    );
    assert!(findings
        .iter()
        .all(|finding| finding.message.contains("does not map")));
}

#[test]
fn explicit_projects_only_requires_explicit_includes() {
    let root = fixture_root("fixture");
    let mut config = load_config(&root);
    config.tests.vitest.projects.clear();
    config.rules[0].options = serde_yaml::from_str("explicitProjectsOnly: true\n").unwrap();
    let result = check_with_files(&root, &config, &[root.join("src/a.test.ts")]);

    let error = result.unwrap_err().to_string();
    assert!(error.contains("explicitProjectsOnly requires at least one tests.vitest.projects"));
}

#[test]
fn explicit_projects_do_not_require_missing_vitest_configs() {
    let root = fixture_root("fixture");
    let mut config = load_config(&root);
    config.tests.vitest.configs = Some(serde_yaml::from_str("missing.config.ts").unwrap());
    config.tests.vitest.projects.insert(
        "custom".to_string(),
        serde_yaml::from_str(
            r#"
include: [src/custom.spec.ts]
"#,
        )
        .unwrap(),
    );
    config.rules[0].options = serde_yaml::from_str("testExtensions: [.spec.ts]\n").unwrap();
    let files = vec![root.join("src/custom.spec.ts")];
    let findings = check_with_files(&root, &config, &files).unwrap();

    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn missing_config_paths_surface_load_errors() {
    let root = fixture_root("fixture");
    let mut config = load_config(&root);
    config.tests.vitest.configs = Some(serde_yaml::from_str("missing.config.ts").unwrap());
    let result = check_with_files(&root, &config, &[root.join("src/a.test.ts")]);

    assert!(result.is_err());
}

#[test]
fn matching_projects_prefers_deepest_config_scope() {
    let projects = vec![
        projects::ProjectGlob {
            name: "root".to_string(),
            explicit: false,
            scope: None,
            include: project_config::build_globset(&["**/*.test.ts".to_string()]).unwrap(),
            exclude: project_config::build_globset(&[]).unwrap(),
        },
        projects::ProjectGlob {
            name: "app".to_string(),
            explicit: false,
            scope: Some("./packages/app".to_string()),
            include: project_config::build_globset(&["packages/app/**/*.test.ts".to_string()])
                .unwrap(),
            exclude: project_config::build_globset(&[]).unwrap(),
        },
    ];

    assert_eq!(
        matching_projects("packages/app/routes.test.ts", &projects),
        vec!["app".to_string()]
    );
}

#[test]
fn matching_projects_keeps_explicit_projects_with_scoped_configs() {
    let projects = vec![
        projects::ProjectGlob {
            name: "explicit".to_string(),
            explicit: true,
            scope: None,
            include: project_config::build_globset(&["packages/app/**/*.test.ts".to_string()])
                .unwrap(),
            exclude: project_config::build_globset(&[]).unwrap(),
        },
        projects::ProjectGlob {
            name: "app".to_string(),
            explicit: false,
            scope: Some("packages/app".to_string()),
            include: project_config::build_globset(&["packages/app/**/*.test.ts".to_string()])
                .unwrap(),
            exclude: project_config::build_globset(&[]).unwrap(),
        },
    ];

    assert_eq!(
        matching_projects("packages/app/routes.test.ts", &projects),
        vec!["explicit".to_string(), "app".to_string()]
    );
}

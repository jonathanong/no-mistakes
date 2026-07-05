use super::*;
use crate::config::v2::{
    schema::{Project, RuleDef, RuleScope},
    NoMistakesConfig,
};

fn fixture_root(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/forbidden-workspace-closure")
            .join(name),
    )
}

fn config(yaml: &str) -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str(yaml).unwrap(),
        ..Default::default()
    });
    config
}

fn package_files(root: &Path, files: &[&str]) -> Vec<PathBuf> {
    files.iter().map(|file| root.join(file)).collect()
}

#[test]
fn reports_direct_external_dependency() {
    let root = fixture_root("direct-external");
    let files = package_files(
        &root,
        &[
            "package.json",
            "packages/app/package.json",
            "packages/domain/package.json",
        ],
    );

    let findings = check_with_files(
        &root,
        &config("packages: [\"@acme/app\"]\nforbidden: [\"@acme/secret\"]\n"),
        &files,
    )
    .unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "packages/app/package.json");
    assert_eq!(findings[0].target.as_deref(), Some("@acme/secret"));
    assert_eq!(
        findings[0].import.as_deref(),
        Some("@acme/app -> @acme/secret")
    );
}

#[test]
fn reports_transitive_external_dependency() {
    let root = fixture_root("transitive-workspace");
    let files = package_files(
        &root,
        &[
            "package.json",
            "packages/app/package.json",
            "packages/domain/package.json",
        ],
    );

    let findings = check_with_files(
        &root,
        &config("packages: [\"@acme/app\"]\nforbidden: [\"@acme/secret\"]\n"),
        &files,
    )
    .unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "packages/domain/package.json");
    assert_eq!(findings[0].target.as_deref(), Some("@acme/secret"));
    assert_eq!(
        findings[0].import.as_deref(),
        Some("@acme/app -> @acme/domain -> @acme/secret")
    );
}

#[test]
fn passes_when_forbidden_package_is_outside_closure() {
    let root = fixture_root("pass");
    let files = package_files(
        &root,
        &[
            "package.json",
            "packages/app/package.json",
            "packages/api/package.json",
            "packages/domain/package.json",
        ],
    );

    let findings = check_with_files(
        &root,
        &config("packages: [\"@acme/app\"]\nforbidden: [\"@acme/secret\"]\n"),
        &files,
    )
    .unwrap();

    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn dependency_types_control_dev_dependency_closure() {
    let root = fixture_root("dependency-types");
    let files = package_files(
        &root,
        &[
            "package.json",
            "packages/app/package.json",
            "packages/domain/package.json",
        ],
    );

    let default_findings = check_with_files(
        &root,
        &config("packages: [\"@acme/app\"]\nforbidden: [\"@acme/secret\"]\n"),
        &files,
    )
    .unwrap();
    assert!(default_findings.is_empty(), "{default_findings:?}");

    let dev_findings = check_with_files(
        &root,
        &config(
            "packages: [\"@acme/app\"]\nforbidden: [\"@acme/secret\"]\ndependencyTypes: [dependencies, devDependencies]\n",
        ),
        &files,
    )
    .unwrap();
    assert_eq!(dev_findings.len(), 1);
    assert_eq!(
        dev_findings[0].import.as_deref(),
        Some("@acme/app -> @acme/domain -> @acme/secret")
    );
}

#[test]
fn glob_pattern_matches_forbidden_package_name() {
    let root = fixture_root("glob");
    let files = package_files(
        &root,
        &[
            "package.json",
            "packages/app/package.json",
            "packages/infra-secret/package.json",
        ],
    );

    let findings = check_with_files(
        &root,
        &config("packages: [\"@acme/app\"]\nforbidden: [\"@acme/*secret\"]\n"),
        &files,
    )
    .unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "packages/app/package.json");
    assert_eq!(findings[0].target.as_deref(), Some("@acme/infra-secret"));
    assert_eq!(
        findings[0].import.as_deref(),
        Some("@acme/app -> @acme/infra-secret")
    );
}

#[test]
fn cycle_does_not_prevent_finding() {
    let root = fixture_root("cycle");
    let files = package_files(
        &root,
        &[
            "package.json",
            "packages/app/package.json",
            "packages/domain/package.json",
        ],
    );

    let findings = check_with_files(
        &root,
        &config("packages: [\"@acme/app\"]\nforbidden: [\"@acme/secret\"]\n"),
        &files,
    )
    .unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].target.as_deref(), Some("@acme/secret"));
    assert_eq!(
        findings[0].import.as_deref(),
        Some("@acme/app -> @acme/domain -> @acme/secret")
    );
}

#[test]
fn manifest_alias_workspace_dependency_extends_closure() {
    let root = fixture_root("manifest-alias-workspace");
    let files = package_files(
        &root,
        &[
            "package.json",
            "packages/app/package.json",
            "packages/domain/package.json",
        ],
    );

    let findings = check_with_files(
        &root,
        &config("packages: [\"@acme/app\"]\nforbidden: [\"@acme/secret\"]\n"),
        &files,
    )
    .unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "packages/domain/package.json");
    assert_eq!(
        findings[0].import.as_deref(),
        Some("@acme/app -> @acme/domain -> @acme/secret")
    );
}

#[test]
fn project_scoped_rule_loads_workspace_from_project_root() {
    let root = fixture_root("project-local-workspace");
    let files = package_files(
        &root,
        &[
            "frontend/pnpm-workspace.yaml",
            "frontend/packages/app/package.json",
            "frontend/packages/domain/package.json",
        ],
    );
    let mut config = config("packages: [\"@acme/app\"]\nforbidden: [\"@acme/secret\"]\n");
    config.projects.insert(
        "frontend".to_string(),
        Project {
            root: Some("frontend".to_string()),
            ..Default::default()
        },
    );
    config.rules[0].scope = None;
    config.rules[0].projects = vec!["frontend".to_string()];

    let findings = check_with_files(&root, &config, &files).unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "frontend/packages/domain/package.json");
    assert_eq!(
        findings[0].import.as_deref(),
        Some("@acme/app -> @acme/domain -> @acme/secret")
    );
}

#[test]
fn invalid_glob_pattern_emits_config_finding() {
    let root = fixture_root("pass");
    let files = package_files(&root, &["package.json", "packages/app/package.json"]);

    let findings = check_with_files(
        &root,
        &config("packages: [\"@acme/app\"]\nforbidden: [\"[\"]\n"),
        &files,
    )
    .unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, ".no-mistakes.yml");
    assert!(findings[0].message.contains("invalid glob pattern"));
}

#[test]
fn unknown_configured_package_emits_config_finding() {
    let root = fixture_root("pass");
    let files = package_files(&root, &["package.json", "packages/app/package.json"]);

    let findings = check_with_files(
        &root,
        &config("packages: [\"@acme/missing\"]\nforbidden: [\"@acme/secret\"]\n"),
        &files,
    )
    .unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, ".no-mistakes.yml");
    assert!(findings[0]
        .message
        .contains("not a named workspace package"));
}

#[test]
fn rule_exclude_filter_suppresses_declaring_manifest() {
    let root = fixture_root("direct-external");
    let files = package_files(
        &root,
        &[
            "package.json",
            "packages/app/package.json",
            "packages/domain/package.json",
        ],
    );
    let mut config = config("packages: [\"@acme/app\"]\nforbidden: [\"@acme/secret\"]\n");
    config.rules[0].exclude = vec!["packages/app/package.json".to_string()];

    let findings = check_with_files(&root, &config, &files).unwrap();

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn traversal_tolerates_duplicate_and_missing_workspace_nodes() {
    let root = fixture_root("pass");
    let mut nodes = BTreeMap::new();
    nodes.insert(
        "@acme/app".to_string(),
        PackageNode {
            manifest: root.join("packages/app/package.json"),
            deps: vec![
                Dependency {
                    name: "@acme/missing".to_string(),
                    resolved_name: None,
                    field: "dependencies".to_string(),
                },
                Dependency {
                    name: "@acme/missing".to_string(),
                    resolved_name: None,
                    field: "dependencies".to_string(),
                },
            ],
        },
    );
    let workspace_names = BTreeSet::from(["@acme/app".to_string(), "@acme/missing".to_string()]);
    let forbidden = build_globset(&["@acme/secret".to_string()]).unwrap();
    let config = config("packages: [\"@acme/app\"]\nforbidden: [\"@acme/secret\"]\n");
    let source_filter =
        crate::codebase::rules::path_filter::RulePathFilter::new(&root, &config, &config.rules[0])
            .unwrap();
    let mut findings = Vec::new();

    traversal::collect_findings_for_package(
        &root,
        "@acme/app",
        &nodes,
        &workspace_names,
        &forbidden,
        &source_filter,
        &mut findings,
    );

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn load_workspace_defaults_to_repository_root_when_target_roots_are_empty() {
    let root = fixture_root("pass");
    let files = package_files(&root, &["package.json", "packages/app/package.json"]);

    let workspace = load_workspace(&root, &[], &files).unwrap();

    assert_eq!(workspace.packages.len(), 1);
    assert_eq!(workspace.packages[0].name, "@acme/app");
}

#[test]
fn normalize_importer_path_keeps_workspace_root_as_dot() {
    assert_eq!(lockfile::normalize_importer_path("./"), ".");
}

#[test]
fn missing_options_emit_config_finding() {
    let root = fixture_root("pass");
    let files = package_files(
        &root,
        &[
            "package.json",
            "packages/app/package.json",
            "packages/domain/package.json",
        ],
    );
    let findings = check_with_files(&root, &config("{}"), &files).unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, ".no-mistakes.yml");
    assert!(findings[0].message.contains("packages"));
    assert!(findings[0].message.contains("forbidden"));
}

use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};

fn fixture_root(name: &str) -> PathBuf {
    normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/production-dependency-declarations")
            .join(name),
    )
}

/// Builds a rule config from `yaml`, always merging in a `workspaceRoots`
/// pointing at the fixture root itself (`workspaceRoots` is required and has
/// no default), so callers only need to spell out the options a scenario
/// actually varies.
fn config(yaml: &str) -> NoMistakesConfig {
    let merged = format!("workspaceRoots: [\".\"]\n{yaml}");
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str(&merged).unwrap(),
        ..Default::default()
    });
    config
}

fn scenario_files(root: &Path, files: &[&str]) -> Vec<PathBuf> {
    files.iter().map(|file| root.join(file)).collect()
}

/// Like `config`, but lets a scenario configure its own `workspaceRoots`
/// instead of the fixture-root default, to test scope-restriction behavior.
fn config_with_roots(roots: &[&str]) -> NoMistakesConfig {
    let quoted: Vec<String> = roots.iter().map(|root| format!("\"{root}\"")).collect();
    let yaml = format!("workspaceRoots: [{}]\n", quoted.join(", "));
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str(&yaml).unwrap(),
        ..Default::default()
    });
    config
}

#[test]
fn dev_only_workspace_import_from_production_file_is_flagged() {
    let root = fixture_root("dev-only-production-import");
    let files = scenario_files(
        &root,
        &[
            "packages/app/package.json",
            "packages/app/index.mts",
            "packages/lib/package.json",
            "packages/lib/index.mts",
            "packages/tool/package.json",
            "packages/tool/index.mts",
        ],
    );

    let findings = check_with_files(&root, &config(""), &files).unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "packages/lib/index.mts");
    assert_eq!(findings[0].target.as_deref(), Some("@acme/tool"));
    assert_eq!(findings[0].import.as_deref(), Some("@acme/tool"));
    assert!(findings[0].message.contains("devDependencies"));
    // The owning package (@acme/lib), not the imported one (@acme/tool), must
    // be named as the package with the missing declaration.
    assert!(findings[0].message.contains("@acme/lib declares only"));
}

#[test]
fn workspace_roots_excludes_an_external_importer_from_reachability_seeding() {
    // Same fixture and files as `dev_only_workspace_import_from_production_file_is_flagged`,
    // which finds a violation when `workspaceRoots` covers the whole fixture
    // root. Here `workspaceRoots` covers only `packages/lib` — the package
    // with the violation — but not `packages/app`, whose import of `@acme/lib`
    // is the only thing that makes `packages/lib/index.mts`
    // production-reachable at all. `workspaceRoots` filters `all_files` before
    // reachability seeding runs, so `packages/app/index.mts` is invisible to
    // the scan and the violation goes unseeded and unflagged.
    let root = fixture_root("dev-only-production-import");
    let files = scenario_files(
        &root,
        &[
            "packages/app/package.json",
            "packages/app/index.mts",
            "packages/lib/package.json",
            "packages/lib/index.mts",
            "packages/tool/package.json",
            "packages/tool/index.mts",
        ],
    );

    let findings = check_with_files(&root, &config_with_roots(&["packages/lib"]), &files).unwrap();

    assert!(findings.is_empty());
}

#[test]
fn test_only_importer_does_not_seed_reachability() {
    let root = fixture_root("test-only-importer");
    let files = scenario_files(
        &root,
        &[
            "packages/app/package.json",
            "packages/app/__tests__/index.test.mts",
            "packages/lib/package.json",
            "packages/lib/index.mts",
        ],
    );

    let findings = check_with_files(&root, &config(""), &files).unwrap();

    assert!(findings.is_empty());
}

#[test]
fn test_pattern_file_reached_via_relative_import_is_excluded() {
    let root = fixture_root("test-pattern-reachable-file-is-excluded");
    let files = scenario_files(
        &root,
        &[
            "packages/app/package.json",
            "packages/app/index.mts",
            "packages/lib/package.json",
            "packages/lib/index.mts",
            "packages/lib/helper.test.mts",
            "packages/tool/package.json",
            "packages/tool/index.mts",
        ],
    );

    // `helper.test.mts` is pulled into the closure by `index.mts`'s relative
    // import, but it matches the default `**/*.test.*` test pattern, so its
    // dev-only `@acme/tool` import must not be flagged.
    let findings = check_with_files(&root, &config(""), &files).unwrap();

    assert!(findings.is_empty());
}

#[test]
fn dependencies_declared_import_passes() {
    let root = fixture_root("dependencies-declared");
    let files = scenario_files(
        &root,
        &[
            "packages/app/package.json",
            "packages/app/index.mts",
            "packages/lib/package.json",
            "packages/lib/index.mts",
        ],
    );

    let findings = check_with_files(&root, &config(""), &files).unwrap();

    assert!(findings.is_empty());
}

#[test]
fn undeclared_import_from_production_file_is_flagged() {
    let root = fixture_root("undeclared-import");
    let files = scenario_files(
        &root,
        &[
            "packages/app/package.json",
            "packages/app/index.mts",
            "packages/lib/package.json",
            "packages/lib/index.mts",
        ],
    );

    let findings = check_with_files(&root, &config(""), &files).unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "packages/lib/index.mts");
    assert_eq!(findings[0].target.as_deref(), Some("left-pad"));
    assert_eq!(findings[0].import.as_deref(), Some("left-pad"));
    assert!(findings[0].message.contains("does not declare"));
    // The owning package (@acme/lib), not the imported one (left-pad), must
    // be named as the package with the missing declaration.
    assert!(findings[0].message.contains("@acme/lib does not declare"));
}

#[test]
fn package_internal_tooling_file_is_not_reachable() {
    let root = fixture_root("package-internal-tooling");
    let files = scenario_files(
        &root,
        &[
            "packages/app/package.json",
            "packages/app/index.mts",
            "packages/lib/package.json",
            "packages/lib/index.mts",
            "packages/lib/scripts/build.mts",
        ],
    );

    let findings = check_with_files(&root, &config(""), &files).unwrap();

    assert!(findings.is_empty());
}

#[test]
fn import_type_of_dev_dependency_is_exempt() {
    let root = fixture_root("import-type-exempt");
    let files = scenario_files(
        &root,
        &[
            "packages/app/package.json",
            "packages/app/index.mts",
            "packages/lib/package.json",
            "packages/lib/index.mts",
        ],
    );

    let findings = check_with_files(&root, &config(""), &files).unwrap();

    assert!(findings.is_empty());
}

#[test]
fn self_reference_subpath_import_is_exempt() {
    let root = fixture_root("self-reference-import");
    let files = scenario_files(
        &root,
        &[
            "packages/app/package.json",
            "packages/app/index.mts",
            "packages/lib/package.json",
            "packages/lib/index.mts",
            "packages/lib/sub.mts",
        ],
    );

    let findings = check_with_files(&root, &config(""), &files).unwrap();

    assert!(findings.is_empty());
}

#[test]
fn third_party_dev_dependency_from_production_is_flagged() {
    let root = fixture_root("third-party-devdependency");
    let files = scenario_files(
        &root,
        &[
            "packages/app/package.json",
            "packages/app/index.mts",
            "packages/lib/package.json",
            "packages/lib/index.mts",
        ],
    );

    let findings = check_with_files(&root, &config(""), &files).unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "packages/lib/index.mts");
    assert_eq!(findings[0].target.as_deref(), Some("chalk"));
    assert_eq!(findings[0].import.as_deref(), Some("chalk"));
}

#[test]
fn relative_import_closure_reaches_a_transitively_imported_file() {
    let root = fixture_root("relative-import-closure");
    let files = scenario_files(
        &root,
        &[
            "packages/app/package.json",
            "packages/app/index.mts",
            "packages/lib/package.json",
            "packages/lib/index.mts",
            "packages/lib/helper.mts",
        ],
    );

    let findings = check_with_files(&root, &config(""), &files).unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "packages/lib/helper.mts");
    assert_eq!(findings[0].target.as_deref(), Some("@acme/tool"));
}

#[test]
fn disable_line_suppression_is_honored() {
    let root = fixture_root("suppression-honored");
    let files = scenario_files(
        &root,
        &[
            "packages/app/package.json",
            "packages/app/index.mts",
            "packages/lib/package.json",
            "packages/lib/index.mts",
        ],
    );

    // The rule's own scan is not suppression-aware in isolation: it still
    // reports the underlying dev-only violation.
    let raw = check_with_files(&root, &config(""), &files).unwrap();
    assert_eq!(raw.len(), 1);
    assert_eq!(raw[0].target.as_deref(), Some("@acme/tool"));

    // The full filesystem dispatch path strips suppressed findings.
    let dispatched =
        crate::codebase::rules::run_filesystem_rules_with_config(&root, &config(""), &files)
            .unwrap();
    assert!(dispatched.is_empty());
}

#[test]
fn workspace_roots_maps_each_configured_relative_path_from_the_root() {
    let root = PathBuf::from("/repo");
    let opts = Options {
        workspace_roots: vec!["packages/lib".to_string(), "packages/app".to_string()],
        allowed_fields: Vec::new(),
        test_file_patterns: Vec::new(),
    };

    let roots = workspace_roots(&root, &opts).unwrap();

    assert_eq!(
        roots,
        vec![
            normalize_path(&root.join("packages/lib")),
            normalize_path(&root.join("packages/app")),
        ]
    );
}

#[test]
fn workspace_roots_rejects_an_unconfigured_empty_value() {
    let root = PathBuf::from("/repo");

    let error = workspace_roots(&root, &Options::default()).unwrap_err();

    assert!(error.contains("workspaceRoots"));
}

#[test]
fn check_with_files_reports_a_config_finding_when_workspace_roots_is_omitted() {
    let root = fixture_root("dependencies-declared");
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str("").unwrap(),
        ..Default::default()
    });

    let findings = check_with_files(&root, &config, &[]).unwrap();

    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("workspaceRoots"));
}

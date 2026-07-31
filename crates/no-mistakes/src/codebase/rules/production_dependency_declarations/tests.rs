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

fn scenario_files(root: &Path, files: &[&str]) -> Vec<PathBuf> {
    files.iter().map(|file| root.join(file)).collect()
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

    let roots = workspace_roots(&root, &opts);

    assert_eq!(
        roots,
        vec![
            normalize_path(&root.join("packages/lib")),
            normalize_path(&root.join("packages/app")),
        ]
    );
}

#[test]
fn workspace_roots_defaults_to_the_check_root_when_unconfigured() {
    let root = PathBuf::from("/repo");

    let roots = workspace_roots(&root, &Options::default());

    assert_eq!(roots, vec![root]);
}

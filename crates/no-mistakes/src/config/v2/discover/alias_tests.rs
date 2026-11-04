use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/config-fragments-aliases")
            .join(name),
    )
}

fn load(name: &str) -> anyhow::Result<super::NoMistakesConfig> {
    let root = fixture(name);
    let files = [
        root.join(".no-mistakes.yml"),
        root.join(".no-mistakes/testPlan/extra.yml"),
    ];
    super::load_v2_config_from_visible(&root, None, &files)
}

#[test]
fn canonical_and_legacy_alias_sections_merge_disjoint_fields() {
    let config = load("disjoint").unwrap();
    let environment = &config.test_plan.vitest.environments["ci"];
    assert_eq!(environment.global_config_fallback, Some(true));
    assert!(environment.groups[0].sample_when_limited);
    let key = crate::config::source_locations::ConfigPath::default()
        .key("testPlan")
        .key("vitest")
        .key("environments")
        .key("ci")
        .key("globalConfigFallback");
    let (path, local) = config.config_provenance.origin(&key).unwrap();
    assert_eq!(path, fixture("disjoint").join(".no-mistakes.yml"));
    let source = std::fs::read_to_string(path).unwrap();
    assert_eq!(
        crate::config::source_locations::ConfigLocations::parse(&source, path)
            .unwrap()
            .line(&local),
        Some(5)
    );
}

#[test]
fn scalar_alias_conflicts_name_both_sources() {
    let error = load("conflict").unwrap_err().to_string();
    assert!(error.contains(".no-mistakes.yml"), "{error}");
    assert!(error.contains("testPlan/extra.yml"), "{error}");
    assert!(error.contains("globalConfigFallback"), "{error}");
}

#[test]
fn deprecated_list_triggers_keep_fragment_paths_and_lines() {
    let root = fixture("deprecated");
    let files = [
        ".no-mistakes.yml",
        ".no-mistakes/checks/extra.yml",
        ".no-mistakes/test_plan/extra.yml",
    ]
    .map(|name| root.join(name));
    let config = super::load_v2_config_from_visible(&root, None, &files).unwrap();
    assert!(config.test_plan.vitest.deprecated_dependencies_key);
    let findings =
        crate::codebase::rules::no_mistakes_config::check_with_files(&root, &config, &files)
            .unwrap();
    let finding = findings
        .iter()
        .find(|finding| finding.message.contains("missing-old.ts"))
        .unwrap();
    assert_eq!(finding.file, ".no-mistakes/test_plan/extra.yml");
    assert_eq!(finding.line, 5);
}

#[test]
fn all_supported_frameworks_retain_deprecated_aliases() {
    let root = fixture("all-frameworks");
    let config = super::load_v2_config(&root, None).unwrap();
    let plan = &config.test_plan;
    let frameworks = [
        &plan.dotnet,
        &plan.playwright,
        &plan.vitest,
        &plan.swift,
        &plan.python,
        &plan.go,
        &plan.cargo,
        &plan.rails,
        &plan.php,
        &plan.java,
        &plan.kotlin,
        &plan.elixir,
        &plan.dart,
        &plan.jest,
    ];
    assert!(frameworks
        .iter()
        .all(|framework| framework.deprecated_dependencies_key));
    assert_eq!(
        plan.vitest.full_suite_triggers.ignore_changed_tests.len(),
        1
    );
    let declared = super::declared_config(&root, None).unwrap();
    assert!(declared["testPlan"]["custom"]
        .get("fullSuiteTriggers")
        .is_some());
}

#[test]
fn same_file_alias_collisions_remain_invalid_in_declared_and_typed_views() {
    let root = fixture("same-file");
    assert!(super::load_v2_config(&root, None).is_err());
    let error = super::declared_config(&root, None).unwrap_err().to_string();
    assert!(error.contains("aliases") && error.contains(".no-mistakes.yml"));
}

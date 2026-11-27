use super::path_globs::validate_playwright_selector_wrappers;
use super::validate_v2_config;
use crate::config::v2::schema::{
    NamedFullSuiteTrigger, NoMistakesConfig, PlaywrightSelectorWrapper, Project, RuleDef,
    TestPlanProjectDependency, TestPlanTargetedProjectDependency,
};
use std::path::{Path, PathBuf};

#[path = "repository_rule_excludes_tests.rs"]
mod repository_rule_excludes_tests;

fn wrapper(module: &str, export: &str, test_id_argument: usize) -> PlaywrightSelectorWrapper {
    PlaywrightSelectorWrapper {
        module: module.to_string(),
        export: export.to_string(),
        test_id_argument,
    }
}

#[test]
fn selector_wrapper_identity_fields_must_not_be_blank() {
    let module_error = validate_playwright_selector_wrappers(&[wrapper(" ", "find", 0)])
        .unwrap_err()
        .to_string();
    assert!(module_error.contains(".module must not be blank"));

    let export_error = validate_playwright_selector_wrappers(&[wrapper("./helpers", " ", 0)])
        .unwrap_err()
        .to_string();
    assert!(export_error.contains(".export must not be blank"));
}

#[test]
fn selector_wrapper_duplicate_arguments_must_not_conflict() {
    validate_playwright_selector_wrappers(&[
        wrapper("./helpers", "find", 1),
        wrapper("./helpers", "find", 1),
    ])
    .unwrap();

    let error = validate_playwright_selector_wrappers(&[
        wrapper("./helpers", "find", 0),
        wrapper("./helpers", "find", 1),
    ])
    .unwrap_err()
    .to_string();
    assert!(error.contains("conflicting testIdArgument values 0 and 1"));
}

#[test]
fn targeted_full_suite_trigger_validates_paths_and_targets() {
    let cases = [
        (
            Vec::new(),
            vec!["unit".to_string()],
            "paths must not be empty",
        ),
        (
            vec!["src/**".to_string()],
            Vec::new(),
            "targets must not be empty",
        ),
        (
            vec!["src/**".to_string()],
            vec![" ".to_string()],
            "targets[0] must not be blank",
        ),
        (
            vec!["[".to_string()],
            vec!["unit".to_string()],
            "contains invalid glob",
        ),
        (
            vec!["!".to_string()],
            vec!["unit".to_string()],
            "paths[0] must not be blank",
        ),
    ];
    for (paths, targets, expected) in cases {
        let mut config = NoMistakesConfig::default();
        config
            .projects
            .insert("app".to_string(), Project::default());
        config.test_plan.vitest.full_suite_triggers.projects.insert(
            "app".to_string(),
            TestPlanProjectDependency::Targeted(TestPlanTargetedProjectDependency {
                paths,
                targets,
                include_changed_tests: None,
            }),
        );
        let error = validate_v2_config(&config, Path::new("config.yml"))
            .unwrap_err()
            .to_string();
        assert!(error.contains(expected), "{error}");
        assert!(error.contains("config.yml.testPlan.vitest.fullSuiteTriggers.projects.app"));
    }

    let mut config = NoMistakesConfig::default();
    config.test_plan.vitest.full_suite_triggers.projects.insert(
        "missing".to_string(),
        TestPlanProjectDependency::Targeted(TestPlanTargetedProjectDependency {
            paths: vec!["src/**".to_string()],
            targets: vec!["unit".to_string()],
            include_changed_tests: None,
        }),
    );
    let error = validate_v2_config(&config, Path::new("config.yml"))
        .unwrap_err()
        .to_string();
    assert!(error.contains(
        "testPlan.vitest.fullSuiteTriggers.projects.missing references missing top-level projects.missing"
    ));

    let mut legacy = NoMistakesConfig::default();
    legacy
        .projects
        .insert("app".to_string(), Project::default());
    legacy
        .test_plan
        .vitest
        .full_suite_triggers
        .projects
        .insert("app".to_string(), TestPlanProjectDependency::All(true));
    validate_v2_config(&legacy, Path::new("config.yml")).unwrap();
}

#[test]
fn project_and_rule_globs_surface_validation_context() {
    let mut project_config = NoMistakesConfig::default();
    project_config.projects.insert(
        "app".to_string(),
        Project {
            include: vec!["[".to_string()],
            ..Default::default()
        },
    );
    let project_error = validate_v2_config(&project_config, Path::new("config.yml"))
        .unwrap_err()
        .to_string();
    assert!(
        project_error.contains("projects.app.include[0]: contains invalid glob `[`"),
        "{project_error}"
    );

    let mut rule_config = NoMistakesConfig::default();
    rule_config.rules.push(RuleDef {
        rule: "unrelated-rule".to_string(),
        exclude: vec!["[".to_string()],
        ..Default::default()
    });
    let rule_error = validate_v2_config(&rule_config, Path::new("config.yml"))
        .unwrap_err()
        .to_string();
    assert!(
        rule_error.contains("rules[0] (unrelated-rule).exclude[0]: contains invalid glob `[`"),
        "{rule_error}"
    );
}

#[test]
fn source_store_config_loading_uses_the_request_snapshot_and_validates_globs() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir.join("../../fixtures/napi/analyze-project-dynamic-import-reachability");
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let visible_paths = snapshot.paths_for(&root);
    let sources = snapshot.source_store_for(&root);

    let config = super::load_v2_config_from_source_store(&root, None, &visible_paths, &sources)
        .expect("fixture config should load from the canonical source store");
    assert_eq!(config.rules.len(), 1);
    assert_eq!(sources.physical_read_count(), 1);

    let invalid_root =
        manifest_dir.join("../../test-cases/config-v2/invalid-rule-path-filter/fixture");
    let error = super::load_v2_config(&invalid_root, None)
        .expect_err("an invalid path glob must be rejected during config validation");
    assert!(error
        .to_string()
        .contains("rules[0] (unique-exports).exclude[0]: contains invalid glob `[`"));
}

#[test]
fn named_triggers_are_validated_for_every_test_plan_framework() {
    let mut config = NoMistakesConfig::default();
    config
        .test_plan
        .python
        .full_suite_triggers
        .triggers
        .push(NamedFullSuiteTrigger {
            name: "resources".to_string(),
            paths: vec!["[".to_string()],
            targets: Vec::new(),
            superseded_by_changes: false,
            include_changed_tests: None,
        });
    let error = validate_v2_config(&config, Path::new("config.yml"))
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("config.yml.testPlan.python.fullSuiteTriggers.triggers[0]"),
        "{error}"
    );
    assert!(error.contains("contains invalid glob"), "{error}");
}

#[test]
fn changed_test_policy_is_rejected_for_programmatic_legacy_named_trigger() {
    let mut config = NoMistakesConfig::default();
    config
        .test_plan
        .vitest
        .full_suite_triggers
        .triggers
        .push(NamedFullSuiteTrigger {
            name: "legacy".to_string(),
            paths: vec!["src/**".to_string()],
            targets: Vec::new(),
            superseded_by_changes: false,
            include_changed_tests: Some(false),
        });
    let error = validate_v2_config(&config, Path::new("config.yml"))
        .unwrap_err()
        .to_string();
    assert!(error.contains(
        "config.yml.testPlan.vitest.fullSuiteTriggers.triggers[0].includeChangedTests requires non-empty targets"
    ));
}

#[test]
fn kotlin_targeted_project_triggers_require_a_top_level_project() {
    let mut config = NoMistakesConfig::default();
    config.test_plan.kotlin.full_suite_triggers.projects.insert(
        "missing".to_string(),
        TestPlanProjectDependency::Targeted(TestPlanTargetedProjectDependency {
            paths: vec!["src/**".to_string()],
            targets: vec!["unit".to_string()],
            include_changed_tests: None,
        }),
    );
    let error = validate_v2_config(&config, Path::new("config.yml"))
        .unwrap_err()
        .to_string();
    assert!(error.contains(
        "testPlan.kotlin.fullSuiteTriggers.projects.missing references missing top-level projects.missing"
    ));
}

#[test]
fn elixir_targeted_project_triggers_require_a_top_level_project() {
    let mut config = NoMistakesConfig::default();
    config.test_plan.elixir.full_suite_triggers.projects.insert(
        "missing".to_string(),
        TestPlanProjectDependency::Targeted(TestPlanTargetedProjectDependency {
            paths: vec!["src/**".to_string()],
            targets: vec!["unit".to_string()],
            include_changed_tests: None,
        }),
    );
    let error = validate_v2_config(&config, Path::new("config.yml"))
        .unwrap_err()
        .to_string();
    assert!(error.contains(
        "testPlan.elixir.fullSuiteTriggers.projects.missing references missing top-level projects.missing"
    ));
}

#[test]
fn dart_targeted_project_triggers_require_a_top_level_project() {
    let mut config = NoMistakesConfig::default();
    config.test_plan.dart.full_suite_triggers.projects.insert(
        "missing".to_string(),
        TestPlanProjectDependency::Targeted(TestPlanTargetedProjectDependency {
            paths: vec!["src/**".to_string()],
            targets: vec!["unit".to_string()],
            include_changed_tests: None,
        }),
    );
    let error = validate_v2_config(&config, Path::new("config.yml"))
        .unwrap_err()
        .to_string();
    assert!(error.contains(
        "testPlan.dart.fullSuiteTriggers.projects.missing references missing top-level projects.missing"
    ));
}

#[test]
fn find_config_root_and_empty_rule_names() {
    assert_eq!(
        super::find_config_root(Path::new("/repo")),
        PathBuf::from("/repo")
    );
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: "  ".to_string(),
        ..Default::default()
    });
    let error = validate_v2_config(&config, Path::new("config.yml"))
        .unwrap_err()
        .to_string();
    assert!(error.contains("rules[0].rule is required"));
}

#[test]
fn fragment_deserialize_without_provenance_uses_fallback_message() {
    let value: serde_yaml::Value = serde_yaml::from_str("rules: bad").unwrap();
    let error = super::fragment_validation::deserialize(
        value,
        &super::super::schema::ConfigProvenance::default(),
        &[],
    )
    .unwrap_err()
    .to_string();
    assert!(
        error.contains("failed to deserialize merged configuration"),
        "{error}"
    );
}

#[test]
fn validate_document_rejects_unknown_rule_key_in_full_document() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/config-v2/rule-keys/fixture/unknown-id.yml");
    let document = super::ConfigDocument {
        source: std::fs::read_to_string(&path).unwrap(),
        path,
        section: None,
    };
    let error = super::fragment_validation::validate_document(&document)
        .unwrap_err()
        .to_string();
    assert!(error.contains("unknown rule"), "{error}");
}

#[test]
fn declared_config_rejects_invalid_rule_options() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/options-audit/fixture/issue-4110");
    let config = root.join("invalid-options.yml");
    let error = super::declared_config(&root, Some(&config))
        .unwrap_err()
        .to_string();
    assert!(error.contains("bogusOption"), "{error}");
}

#[test]
fn fragment_validation_drops_location_when_the_origin_is_unrecorded() {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: "some-rule".to_string(),
        projects: vec!["missing".to_string()],
        ..RuleDef::default()
    });
    let missing = super::fragment_validation::validate(
        &config,
        &crate::config::v2::schema::ConfigProvenance::default(),
        &[],
        Path::new("config.yml"),
    )
    .unwrap_err()
    .to_string();
    assert!(
        missing.contains("undeclared project `missing`"),
        "{missing}"
    );

    let mut provenance = crate::config::v2::schema::ConfigProvenance::default();
    provenance.insert(
        crate::config::source_locations::ConfigPath::default()
            .key("rules")
            .index(0)
            .key("projects")
            .index(0),
        PathBuf::from("other.yml"),
        crate::config::source_locations::ConfigPath::default(),
    );
    let elsewhere =
        super::fragment_validation::validate(&config, &provenance, &[], Path::new("config.yml"))
            .unwrap_err()
            .to_string();
    assert!(
        elsewhere.contains("undeclared project `missing`"),
        "{elsewhere}"
    );

    let mut unparsed_provenance = crate::config::v2::schema::ConfigProvenance::default();
    unparsed_provenance.insert(
        crate::config::source_locations::ConfigPath::default()
            .key("rules")
            .index(0)
            .key("projects")
            .index(0),
        PathBuf::from("other"),
        crate::config::source_locations::ConfigPath::default(),
    );
    let documents = [super::ConfigDocument {
        path: PathBuf::from("other"),
        section: None,
        source: "rules: {}\n".to_string(),
    }];
    let unparsed = super::fragment_validation::validate(
        &config,
        &unparsed_provenance,
        &documents,
        Path::new("config.yml"),
    )
    .unwrap_err()
    .to_string();
    assert!(
        unparsed.contains("undeclared project `missing`"),
        "{unparsed}"
    );
}

#[test]
fn fragment_finish_keeps_setup_files_when_provenance_is_missing() {
    let mut config = NoMistakesConfig::default();
    config.tests.vitest.projects.insert(
        "web".to_string(),
        crate::config::v2::schema::TestProjectPolicy {
            include: vec!["src/**".to_string()],
            setup_files: vec!["setup.ts".to_string()],
            ..crate::config::v2::schema::TestProjectPolicy::default()
        },
    );
    super::fragment_validation::finish(Path::new("."), &mut config, &[], Path::new("config.yml"))
        .unwrap();
    let policy = &config.tests.vitest.projects["web"];
    let recorded = policy
        .setup_declaration_paths
        .first()
        .expect("setup file path");
    assert!(recorded.ends_with("config.yml"), "{recorded:?}");
    assert_eq!(policy.declaration_path.as_deref(), Some(recorded.as_path()));

    let mut bare = NoMistakesConfig::default();
    bare.tests.vitest.projects.insert(
        "api".to_string(),
        crate::config::v2::schema::TestProjectPolicy::default(),
    );
    super::fragment_validation::finish(Path::new("."), &mut bare, &[], Path::new("config.yml"))
        .unwrap();
    let bare_policy = &bare.tests.vitest.projects["api"];
    assert!(bare_policy.setup_declaration_paths.is_empty());
    assert!(bare_policy.declaration_path.is_none());
}

#[test]
fn fragment_deserialize_ignores_provenance_for_another_file() {
    let value: serde_yaml::Value = serde_yaml::from_str("rules: bad").unwrap();
    let mut provenance = crate::config::v2::schema::ConfigProvenance::default();
    provenance.insert(
        crate::config::source_locations::ConfigPath::default().key("rules"),
        PathBuf::from("other.yml"),
        crate::config::source_locations::ConfigPath::default(),
    );
    let documents = [super::ConfigDocument {
        path: PathBuf::from("root.yml"),
        section: None,
        source: "rules: bad\n".to_string(),
    }];
    let error = super::fragment_validation::deserialize(value, &provenance, &documents)
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("failed to deserialize merged configuration"),
        "{error}"
    );
}

#[test]
fn empty_change_category_test_glob_fails_at_load() {
    let mut config = NoMistakesConfig::default();
    config.changes.categories.insert(
        "api".to_string(),
        crate::changes::config::ChangeCategory {
            paths: vec!["src/**".to_string()],
            tests: vec![String::new()],
            ..crate::changes::config::ChangeCategory::default()
        },
    );
    let error = validate_v2_config(&config, Path::new("config.yml"))
        .unwrap_err()
        .to_string();
    assert!(error.contains("empty glob"), "{error}");
}

use super::*;
use crate::config::v2::NoMistakesConfig;
use std::path::PathBuf;

fn v2_config_fixture(name: &str) -> NoMistakesConfig {
    let yaml = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/config-v2")
            .join(name)
            .join("fixture")
            .join(".no-mistakes.yml"),
    )
    .unwrap();
    serde_yaml::from_str(&yaml).unwrap()
}

#[test]
fn rule_enabled_defaults_true_and_reads_false() {
    let config = Config::from_yaml(
        r#"
rules:
  default-rule: {}
  disabled-rule:
    enabled: false
"#,
    )
    .unwrap();

    assert!(config.is_rule_enabled("missing-rule"));
    assert!(config.is_rule_enabled("default-rule"));
    assert!(!config.is_rule_enabled("disabled-rule"));
}

#[test]
fn augment_from_gitignore_adds_plain_directory_names_once() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/ast-snippets/config/fixture/gitignore-project");
    let mut config = Config {
        filesystem: FilesystemConfig {
            skip_directories: vec!["dist".to_string()],
        },
        projects: HashMap::new(),
        repository_rules: HashSet::new(),
        rules: HashMap::new(),
        rule_applications: Vec::new(),
    };

    config.augment_from_gitignore(&root);

    assert_eq!(
        config.filesystem.skip_directories,
        vec!["dist".to_string(), "node_modules".to_string()]
    );
}

#[test]
fn augment_from_gitignore_ignores_missing_file() {
    let mut config = Config::default();

    config.augment_from_gitignore(Path::new("/no/such/project"));

    assert!(config.filesystem.skip_directories.is_empty());
}

#[test]
fn load_codebase_config_uses_explicit_config_path() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/config-v2/disabled-rule/fixture");
    let config_path = root.join(".no-mistakes.yml");

    let config = load_codebase_config_with_path(&root, Some(&config_path)).unwrap();

    assert!(config.is_rule_enabled("active-rule"));
    assert!(!config.is_rule_enabled("disabled-rule"));
}

#[test]
fn load_codebase_config_defaults_when_no_config_exists() {
    let root = tempfile::tempdir().unwrap();

    let config = load_codebase_config_with_path(root.path(), None).unwrap();

    assert!(config.filesystem.skip_directories.is_empty());
    assert!(config.projects.is_empty());
    assert!(config.rules.is_empty());
}

#[test]
fn load_config_with_explicit_config_uses_config_parent_gitignore() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/config-v2/explicit-config-parent/fixture");
    let nested = root.join("nested");

    let config = load_config_with_path(&nested, Some(Path::new("../.no-mistakes.yml"))).unwrap();

    assert_eq!(config.filesystem.skip_directories, vec!["from-config"]);
}

#[test]
fn load_codebase_config_finds_parent_guardrails_config() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/codebase-intel/fixture");
    let nested = root.join("packages/api/src");

    let config = load_codebase_config_with_path(&nested, None).unwrap();
    let routes: RouteOptions = config.rule_options("route-consistency");

    assert_eq!(routes.backend_pattern, "packages/api/src/**/*.mts");
    assert_eq!(routes.frontend_root, "packages/web/app");
}

#[test]
fn load_codebase_config_finds_parent_no_mistakes_config() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/unique-exports-config-disabled/fixture");
    let nested = root.join("src/nested");

    let config = load_codebase_config_with_path(&nested, None).unwrap();

    assert!(!config.is_rule_enabled("unique-exports"));
}

#[test]
fn v2_duplicate_rule_applications_enable_rule_when_any_application_is_enabled() {
    let config = v2_config_fixture("duplicate-rule-applications");

    let config = conversion::config_from_v2(config);

    assert!(config.is_rule_enabled("unique-exports"));
    assert_eq!(
        config.project_roots_for_rule(Path::new("/repo"), "unique-exports"),
        vec![PathBuf::from("/repo/web")]
    );
}

#[test]
fn v2_rule_applications_preserve_per_application_options() {
    let config = v2_config_fixture("multiple-rule-application-options");

    let config = conversion::config_from_v2(config);
    let options = config
        .rule_applications_for("rust-max-lines-per-file")
        .into_iter()
        .map(|application| {
            application.rule_options::<super::super::rules::rust_max_lines_per_file::Options>()
        })
        .map(|options| options.src_max)
        .collect::<Vec<_>>();

    assert_eq!(options, vec![Some(100), Some(80)]);
}

#[test]
fn v2_rule_application_project_without_root_uses_workspace_root() {
    let config = v2_config_fixture("rule-application-default-root");

    let config = conversion::config_from_v2(config);
    let applications = config.rule_applications_for("unique-exports");

    assert_eq!(applications.len(), 1);
    assert_eq!(
        config.project_roots_for_rule_application(Path::new("/repo"), applications[0]),
        vec![PathBuf::from("/repo")]
    );
}

#[test]
fn v2_repository_rule_application_keeps_workspace_root_with_project_targets() {
    let config = v2_config_fixture("repository-and-project-rule");

    let config = conversion::config_from_v2(config);

    assert_eq!(
        config.project_roots_for_rule(Path::new("/repo"), "unique-exports"),
        vec![PathBuf::from("/repo"), PathBuf::from("/repo/web")]
    );
}

#[test]
fn v2_unknown_rule_project_targets_are_ignored() {
    let config = v2_config_fixture("unknown-rule-project-target");

    let config = conversion::config_from_v2(config);

    assert!(config
        .project_roots_for_rule(Path::new("/repo"), "unique-exports")
        .is_empty());
}

#[test]
fn v2_untargeted_enabled_rules_are_not_converted_to_global_codebase_rules() {
    let config = v2_config_fixture("untargeted-enabled-rule");

    let config = conversion::config_from_v2(config);

    assert!(!config.rules.contains_key("unique-exports"));
    assert!(config
        .project_roots_for_rule(Path::new("/repo"), "unique-exports")
        .is_empty());
}

#[test]
fn load_codebase_config_rejects_duplicate_parent_configs() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/config-v2/duplicate-stems/fixture");

    let error = load_codebase_config_with_path(&root, None).unwrap_err();

    assert!(error.to_string().contains("multiple config files found"));
}

#[test]
fn project_roots_for_rule_covers_default_and_unmatched_projects() {
    let root = Path::new("/repo");
    let config = Config::from_yaml(
        r#"
projects:
  app:
    rules: [unique-exports]
  other:
    rules: [different-rule]
"#,
    )
    .unwrap();

    assert_eq!(
        config.project_roots_for_rule(root, "unique-exports"),
        vec![PathBuf::from("/repo")]
    );
    assert!(config
        .project_roots_for_rule(root, "missing-rule")
        .is_empty());
}

#[test]
fn project_roots_for_rule_infers_nextjs_root() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/config-v2/nextjs-inferred-root/fixture");
    let root = crate::codebase::ts_resolver::normalize_path(&root);

    let mut config = load_codebase_config_with_path(&root, None).unwrap();
    config.projects.insert(
        "marketing".to_string(),
        project::ProjectConfig {
            type_: Some(crate::config::v2::schema::ProjectType::Nextjs),
            rules: vec!["unique-exports".to_string()],
            ..Default::default()
        },
    );

    assert_eq!(
        config.project_roots_for_rule(&root, "unique-exports"),
        vec![root.join("web")]
    );
}

#[test]
fn project_roots_for_rule_infers_remix_root() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/config-v2/remix-inferred-root/fixture");
    let root = crate::codebase::ts_resolver::normalize_path(&root);

    let mut config = load_codebase_config_with_path(&root, None).unwrap();
    config.projects.insert(
        "marketing".to_string(),
        project::ProjectConfig {
            type_: Some(crate::config::v2::schema::ProjectType::Remix),
            rules: vec!["unique-exports".to_string()],
            ..Default::default()
        },
    );

    assert_eq!(
        config.project_roots_for_rule(&root, "unique-exports"),
        vec![root.join("web")]
    );
}

#[test]
fn project_config_effective_root_infers_remix_root() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/config-v2/remix-inferred-root/fixture");
    let root = crate::codebase::ts_resolver::normalize_path(&root);
    let project = project::ProjectConfig {
        type_: Some(crate::config::v2::schema::ProjectType::Remix),
        ..Default::default()
    };

    assert_eq!(project.effective_root(&root), Some(root.join("web")));
}

#[test]
fn project_roots_for_rule_infers_remix_vite_root() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/config-v2/remix-vite-inferred-root/fixture");
    let root = crate::codebase::ts_resolver::normalize_path(&root);

    let mut config = load_codebase_config_with_path(&root, None).unwrap();
    config.projects.insert(
        "marketing".to_string(),
        project::ProjectConfig {
            type_: Some(crate::config::v2::schema::ProjectType::Remix),
            rules: vec!["unique-exports".to_string()],
            ..Default::default()
        },
    );

    assert_eq!(
        config.project_roots_for_rule(&root, "unique-exports"),
        vec![root.join("web")]
    );
}

#[test]
fn project_config_effective_root_infers_remix_vite_root() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/config-v2/remix-vite-inferred-root/fixture");
    let root = crate::codebase::ts_resolver::normalize_path(&root);
    let project = project::ProjectConfig {
        type_: Some(crate::config::v2::schema::ProjectType::Remix),
        ..Default::default()
    };

    assert_eq!(project.effective_root(&root), Some(root.join("web")));
}

#[test]
fn project_roots_for_rule_infers_vitejs_root() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/config-v2/vitejs-inferred-root/fixture");
    let root = crate::codebase::ts_resolver::normalize_path(&root);

    let mut config = load_codebase_config_with_path(&root, None).unwrap();
    config.projects.insert(
        "marketing".to_string(),
        project::ProjectConfig {
            type_: Some(crate::config::v2::schema::ProjectType::Vitejs),
            rules: vec!["unique-exports".to_string()],
            ..Default::default()
        },
    );

    assert_eq!(
        config.project_roots_for_rule(&root, "unique-exports"),
        vec![root.join("web")]
    );
}

#[test]
fn project_config_effective_root_infers_vitejs_root() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/config-v2/vitejs-inferred-root/fixture");
    let root = crate::codebase::ts_resolver::normalize_path(&root);
    let project = project::ProjectConfig {
        type_: Some(crate::config::v2::schema::ProjectType::Vitejs),
        ..Default::default()
    };

    assert_eq!(project.effective_root(&root), Some(root.join("web")));
}

#[test]
fn project_config_effective_root_infers_nextjs_root() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/config-v2/nextjs-inferred-root/fixture");
    let root = crate::codebase::ts_resolver::normalize_path(&root);
    let project = project::ProjectConfig {
        type_: Some(crate::config::v2::schema::ProjectType::Nextjs),
        ..Default::default()
    };

    assert_eq!(project.effective_root(&root), Some(root.join("web")));
}

#[test]
fn project_roots_for_rule_falls_back_when_nextjs_root_is_not_inferred() {
    let root = Path::new("/repo");
    let mut projects = HashMap::new();
    projects.insert(
        "web".to_string(),
        project::ProjectConfig {
            type_: Some(crate::config::v2::schema::ProjectType::Nextjs),
            rules: vec!["unique-exports".to_string()],
            ..Default::default()
        },
    );

    assert_eq!(
        project::roots_for_rule(
            &projects,
            &HashMap::new(),
            &HashSet::new(),
            root,
            "unique-exports"
        ),
        vec![PathBuf::from("/repo")]
    );
}

#[test]
fn project_roots_for_rule_falls_back_when_remix_root_is_not_inferred() {
    let root = Path::new("/repo");
    let mut projects = HashMap::new();
    projects.insert(
        "web".to_string(),
        project::ProjectConfig {
            type_: Some(crate::config::v2::schema::ProjectType::Remix),
            rules: vec!["unique-exports".to_string()],
            ..Default::default()
        },
    );

    assert_eq!(
        project::roots_for_rule(
            &projects,
            &HashMap::new(),
            &HashSet::new(),
            root,
            "unique-exports"
        ),
        vec![PathBuf::from("/repo")]
    );
}

#[test]
fn project_roots_for_rule_falls_back_when_vitejs_root_is_not_inferred() {
    let root = Path::new("/repo");
    let mut projects = HashMap::new();
    projects.insert(
        "web".to_string(),
        project::ProjectConfig {
            type_: Some(crate::config::v2::schema::ProjectType::Vitejs),
            rules: vec!["unique-exports".to_string()],
            ..Default::default()
        },
    );

    assert_eq!(
        project::roots_for_rule(
            &projects,
            &HashMap::new(),
            &HashSet::new(),
            root,
            "unique-exports"
        ),
        vec![PathBuf::from("/repo")]
    );
}

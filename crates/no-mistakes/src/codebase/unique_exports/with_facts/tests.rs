use super::helpers::{filter_application_files, relative, shared_symbol_files};
use crate::codebase::config::{Config, ProjectConfig, RuleApplicationConfig};
use std::collections::{HashMap, HashSet};
use std::path::Path;

fn config_with_projects(projects: HashMap<String, ProjectConfig>) -> Config {
    Config {
        projects,
        rules: HashMap::new(),
        repository_rules: HashSet::new(),
        rule_applications: Vec::new(),
        ..Default::default()
    }
}

#[test]
fn filter_application_files_matches_repository_rule_filters() {
    let root = Path::new("/repo");
    let config = Config::default();
    let application = RuleApplicationConfig {
        repository: true,
        include: vec!["src/**".to_string()],
        exclude: vec!["src/generated/**".to_string()],
        ..Default::default()
    };
    let files = vec![
        root.join("src/app.ts"),
        root.join("src/generated/app.ts"),
        root.join("test/app.ts"),
    ];

    let filtered = filter_application_files(root, &config, &application, files, None).unwrap();

    assert_eq!(filtered, vec![root.join("src/app.ts")]);
}

#[test]
fn filter_application_files_matches_project_and_rule_relative_filters() {
    let root = Path::new("/repo");
    let config = config_with_projects(
        [(
            "web".to_string(),
            ProjectConfig {
                root: Some("web".to_string()),
                include: vec!["src/**".to_string()],
                exclude: vec!["**/*.stories.tsx".to_string()],
                ..Default::default()
            },
        )]
        .into_iter()
        .collect(),
    );
    let application = RuleApplicationConfig {
        projects: vec!["missing".to_string(), "web".to_string()],
        include: vec!["src/**/*.tsx".to_string()],
        exclude: vec!["generated/**".to_string()],
        ..Default::default()
    };
    let files = vec![
        root.join("web/src/Button.tsx"),
        root.join("web/src/Button.stories.tsx"),
        root.join("web/generated/Button.tsx"),
        root.join("web/test/Button.tsx"),
    ];

    let filtered = filter_application_files(root, &config, &application, files, None).unwrap();

    assert_eq!(filtered, vec![root.join("web/src/Button.tsx")]);
}

#[test]
fn filter_application_files_falls_back_when_project_root_cannot_be_inferred() {
    let root = Path::new("/repo");
    let config = config_with_projects(
        [(
            "unrooted".to_string(),
            ProjectConfig {
                include: vec!["src/**".to_string()],
                ..Default::default()
            },
        )]
        .into_iter()
        .collect(),
    );
    let application = RuleApplicationConfig {
        projects: vec!["unrooted".to_string()],
        ..Default::default()
    };
    let files = vec![root.join("src/app.ts"), root.join("lib/util.ts")];

    let filtered = filter_application_files(root, &config, &application, files, None).unwrap();

    assert_eq!(filtered, vec![root.join("src/app.ts")]);
}

#[test]
fn filter_application_files_skips_invalid_project_filters() {
    let root = Path::new("/repo");
    let config = config_with_projects(
        [
            (
                "bad-include".to_string(),
                ProjectConfig {
                    root: Some("bad-include".to_string()),
                    include: vec!["[".to_string()],
                    ..Default::default()
                },
            ),
            (
                "bad-exclude".to_string(),
                ProjectConfig {
                    root: Some("bad-exclude".to_string()),
                    exclude: vec!["[".to_string()],
                    ..Default::default()
                },
            ),
        ]
        .into_iter()
        .collect(),
    );
    let application = RuleApplicationConfig {
        projects: vec!["bad-include".to_string(), "bad-exclude".to_string()],
        ..Default::default()
    };
    let files = vec![
        root.join("bad-include/src/app.ts"),
        root.join("bad-exclude/src/app.ts"),
    ];

    let filtered = filter_application_files(root, &config, &application, files, None).unwrap();

    assert!(filtered.is_empty());
}

#[test]
fn filter_application_files_reports_invalid_rule_filters() {
    let root = Path::new("/repo");
    let config = Config::default();
    let application = RuleApplicationConfig {
        repository: true,
        include: vec!["[".to_string()],
        ..Default::default()
    };

    let error = filter_application_files(
        root,
        &config,
        &application,
        vec![root.join("src/app.ts")],
        None,
    )
    .unwrap_err();

    assert!(error
        .to_string()
        .contains("unique-exports rule include contains invalid glob"));

    let application = RuleApplicationConfig {
        repository: true,
        exclude: vec!["[".to_string()],
        ..Default::default()
    };

    let error = filter_application_files(
        root,
        &config,
        &application,
        vec![root.join("src/app.ts")],
        None,
    )
    .unwrap_err();

    assert!(error
        .to_string()
        .contains("unique-exports rule exclude contains invalid glob"));
}

#[test]
fn relative_keeps_paths_outside_the_root() {
    assert_eq!(
        relative(Path::new("/repo"), Path::new("/other/app.ts")),
        "/other/app.ts"
    );
}

#[test]
fn shared_symbol_files_dedupes_workspace_and_analysis_paths() {
    let files = shared_symbol_files(
        &[Path::new("/repo/src/a.ts").to_path_buf()],
        &[
            Path::new("/repo/src/a.ts").to_path_buf(),
            Path::new("/repo/src/b.ts").to_path_buf(),
        ],
    );
    assert_eq!(
        files,
        vec![
            Path::new("/repo/src/a.ts").to_path_buf(),
            Path::new("/repo/src/b.ts").to_path_buf()
        ]
    );
}

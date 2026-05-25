use super::*;
use crate::config::v2::schema::{Project, RuleDef, RuleTestTargets};

fn config() -> NoMistakesConfig {
    NoMistakesConfig {
        projects: [(
            "web".to_string(),
            Project {
                root: Some("web".to_string()),
                include: vec!["src/**".to_string()],
                exclude: vec!["**/*.stories.tsx".to_string()],
                ..Default::default()
            },
        )]
        .into_iter()
        .collect(),
        ..Default::default()
    }
}

#[test]
fn project_and_rule_filters_match_repo_and_project_relative_paths() {
    let root = Path::new("/repo");
    let config = config();
    let rule = RuleDef {
        rule: "unique-exports".to_string(),
        projects: vec!["web".to_string()],
        include: vec!["src/**/*.tsx".to_string()],
        exclude: vec!["generated/**".to_string()],
        ..Default::default()
    };

    let filter = RulePathFilter::new(root, &config, &rule).unwrap();

    assert!(filter.is_match(Path::new("/repo/web/src/Button.tsx")));
    assert!(!filter.is_match(Path::new("/repo/web/src/Button.stories.tsx")));
    assert!(!filter.is_match(Path::new("/repo/web/generated/Button.tsx")));
    assert!(!filter.is_match(Path::new("/repo/web/test/Button.tsx")));
}

#[test]
fn empty_rule_include_matches_all_files_in_target_project() {
    let root = Path::new("/repo");
    let config = config();
    let rule = RuleDef {
        rule: "rust-max-lines-per-file".to_string(),
        projects: vec!["web".to_string()],
        ..Default::default()
    };

    let filter = RulePathFilter::new(root, &config, &rule).unwrap();

    assert!(filter.is_match(Path::new("/repo/web/src/lib.ts")));
    assert!(!filter.is_match(Path::new("/repo/backend/src/lib.ts")));
}

#[test]
fn unknown_project_targets_are_ignored() {
    let root = Path::new("/repo");
    let config = config();
    let rule = RuleDef {
        rule: "rust-max-lines-per-file".to_string(),
        projects: vec!["missing".to_string(), "web".to_string()],
        ..Default::default()
    };

    let filter = RulePathFilter::new(root, &config, &rule).unwrap();

    assert!(filter.is_match(Path::new("/repo/web/src/lib.ts")));
}

#[test]
fn repository_rule_rejects_paths_outside_root() {
    let root = Path::new("/repo");
    let config = NoMistakesConfig::default();
    let rule = RuleDef {
        rule: "rust-max-lines-per-file".to_string(),
        scope: Some(crate::config::v2::schema::RuleScope::Repository),
        ..Default::default()
    };

    let filter = RulePathFilter::new(root, &config, &rule).unwrap();

    assert!(filter.is_match(Path::new("/repo/src/lib.rs")));
    assert!(filter.is_match(Path::new("src/lib.rs")));
    assert!(!filter.is_match(Path::new("/outside/src/lib.rs")));
}

#[test]
fn test_targeted_rule_matches_repository_paths() {
    let root = Path::new("/repo");
    let config = NoMistakesConfig::default();
    let rule = RuleDef {
        rule: "playwright-unique-test-ids".to_string(),
        tests: RuleTestTargets {
            playwright: vec!["chromium".to_string()],
            ..Default::default()
        },
        include: vec!["tests/**".to_string()],
        ..Default::default()
    };

    let filter = RulePathFilter::new(root, &config, &rule).unwrap();

    assert!(filter.is_match(Path::new("/repo/tests/login.spec.ts")));
    assert!(!filter.is_match(Path::new("/repo/src/app.ts")));
}

#[test]
fn invalid_project_include_reports_context() {
    let root = Path::new("/repo");
    let mut config = config();
    config
        .projects
        .get_mut("web")
        .unwrap()
        .include
        .push("[".to_string());
    let rule = RuleDef {
        rule: "rust-max-lines-per-file".to_string(),
        projects: vec!["web".to_string()],
        ..Default::default()
    };

    let error = match RulePathFilter::new(root, &config, &rule) {
        Ok(_) => panic!("expected invalid project include error"),
        Err(error) => error,
    };

    assert!(error
        .to_string()
        .contains("project `web` include contains invalid glob"));
}

#[test]
fn invalid_project_exclude_reports_context() {
    let root = Path::new("/repo");
    let mut config = config();
    config
        .projects
        .get_mut("web")
        .unwrap()
        .exclude
        .push("[".to_string());
    let rule = RuleDef {
        rule: "rust-max-lines-per-file".to_string(),
        projects: vec!["web".to_string()],
        ..Default::default()
    };

    let error = match RulePathFilter::new(root, &config, &rule) {
        Ok(_) => panic!("expected invalid project exclude error"),
        Err(error) => error,
    };

    assert!(error
        .to_string()
        .contains("project `web` exclude contains invalid glob"));
}

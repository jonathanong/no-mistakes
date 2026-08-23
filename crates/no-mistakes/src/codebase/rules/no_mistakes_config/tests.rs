use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/no-mistakes-config/fixture")
            .join(name),
    )
}

fn enable(mut config: NoMistakesConfig) -> NoMistakesConfig {
    config.rules.insert(
        0,
        RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            ..Default::default()
        },
    );
    config
}

fn load(root: &Path) -> NoMistakesConfig {
    let yaml = std::fs::read_to_string(root.join(".no-mistakes.yml")).unwrap();
    enable(serde_yaml::from_str(&yaml).unwrap())
}

fn files(root: &Path) -> Vec<PathBuf> {
    let mut files = vec![root.join(".no-mistakes.yml")];
    for name in [
        "web/src/index.ts",
        "vitest.config.ts",
        "playwright/tests/a.spec.mts",
    ] {
        let path = root.join(name);
        if path.exists() {
            files.push(path);
        }
    }
    files
}

fn run(name: &str) -> Vec<RuleFinding> {
    let root = fixture(name);
    check_with_files(&root, &load(&root), &files(&root)).unwrap()
}

#[test]
fn missing_project_root_is_a_finding() {
    let findings = run("missing-path");
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("projects.web.root")
                && finding.message.contains("missing")),
        "{findings:?}"
    );
}

#[test]
fn empty_exclude_glob_is_a_finding() {
    let findings = run("empty-glob");
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("matches no tracked files")),
        "{findings:?}"
    );
}

#[test]
fn env_level_limit_with_direct_group_is_a_finding() {
    let findings = run("env-limit");
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("#9440")),
        "{findings:?}"
    );
}

#[test]
fn env_all_with_limit_is_not_a_direct_group_conflict() {
    let root = fixture("pass");
    let config = enable(
        serde_yaml::from_str(
            "testPlan:\n  vitest:\n    environments:\n      prePush:\n        all: true\n        limit:\n          files: 10\n",
        )
        .unwrap(),
    );
    let findings = check_with_files(&root, &config, &files(&root)).unwrap();
    assert!(
        findings
            .iter()
            .all(|finding| !finding.message.contains("#9440")),
        "{findings:?}"
    );
}

#[test]
fn omitted_groups_still_flag_env_level_limit() {
    let root = fixture("pass");
    let config = enable(
        serde_yaml::from_str(
            "testPlan:\n  vitest:\n    environments:\n      prePush:\n        limit:\n          files: 10\n",
        )
        .unwrap(),
    );
    let findings = check_with_files(&root, &config, &files(&root)).unwrap();
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("#9440")),
        "{findings:?}"
    );
}

#[test]
fn valid_config_is_silent() {
    assert!(run("pass").is_empty(), "{:?}", run("pass"));
}

#[test]
fn disabled_rule_is_silent() {
    let root = fixture("missing-path");
    let config: NoMistakesConfig =
        serde_yaml::from_str(&std::fs::read_to_string(root.join(".no-mistakes.yml")).unwrap())
            .unwrap();
    assert!(check_with_files(&root, &config, &files(&root))
        .unwrap()
        .is_empty());
}

#[test]
fn repo_root_dot_is_a_present_directory() {
    let root = fixture("pass");
    let config = enable(serde_yaml::from_str("projects:\n  app:\n    root: .\n").unwrap());
    assert!(
        check_with_files(&root, &config, &files(&root))
            .unwrap()
            .iter()
            .all(|finding| !finding.message.contains("projects.app.root")),
        "{:?}",
        check_with_files(&root, &config, &files(&root)).unwrap()
    );
}

#[test]
fn percent_limit_and_runner_paths_are_linted() {
    let root = fixture("pass");
    let config = enable(
        serde_yaml::from_str(
            r#"
frontendRoot: web
projects:
  web:
    root: web
    include: ["web/src/**"]
    exclude: ["no-such-dir/**"]
tests:
  playwright:
    configs: [vitest.config.ts, ""]
    selectorRoots: [web]
    frontendRoot: web
    navigationHelpers: [vitest.config.ts]
  vitest:
    configs: vitest.config.ts
  jest:
    configs: vitest.config.ts
  storybook:
    configs: vitest.config.ts
  swift:
    packages: [web]
testPlan:
  vitest:
    fullSuiteTriggers:
      - name: sources
        paths:
          - web/src/**
          - vitest.config.ts
    environments:
      prePush:
        include: ["web/src/**"]
        exclude: [""]
        limit:
          percent: 40
"#,
        )
        .unwrap(),
    );
    let findings = check_with_files(&root, &config, &files(&root)).unwrap();
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("no-such-dir")
                && finding.message.contains("matches no tracked files")),
        "{findings:?}"
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("#9440")),
        "{findings:?}"
    );
}

#[test]
fn missing_file_and_glob_paths_use_kind_labels() {
    let root = fixture("pass");
    let config = enable(
        serde_yaml::from_str(
            r#"
testPlan:
  vitest:
    fullSuiteTriggers:
      - name: missing
        paths:
          - no-such-file.ts
          - no-such-glob/**
"#,
        )
        .unwrap(),
    );
    let findings = check_with_files(&root, &config, &files(&root)).unwrap();
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("missing file `no-such-file.ts`")),
        "{findings:?}"
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("missing path `no-such-glob/**`")),
        "{findings:?}"
    );
}

#[test]
fn invalid_include_and_exclude_globs_error() {
    let root = fixture("pass");
    let yaml_cases = [
        "testPlan:\n  vitest:\n    environments:\n      prePush:\n        include:\n          - \"{\"\n",
        "testPlan:\n  vitest:\n    environments:\n      prePush:\n        exclude:\n          - \"{\"\n",
        "projects:\n  web:\n    root: web\n    include:\n      - \"{\"\n",
        "projects:\n  web:\n    root: web\n    exclude:\n      - \"{\"\n",
    ];
    for yaml in yaml_cases {
        let config = enable(serde_yaml::from_str(yaml).unwrap());
        assert!(
            check_with_files(&root, &config, &files(&root)).is_err(),
            "{yaml}"
        );
    }
    let mut config = enable(serde_yaml::from_str("projects:\n  web:\n    root: web\n").unwrap());
    config.rules[0].include = vec!["{".to_string()];
    assert!(check_with_files(&root, &config, &files(&root)).is_err());
    config.rules[0].include.clear();
    config.rules[0].exclude = vec!["{".to_string()];
    assert!(check_with_files(&root, &config, &files(&root)).is_err());
}

#[test]
fn unconfigured_rule_reports_nothing() {
    let root = fixture("missing-path");
    let findings = check_with_files(&root, &NoMistakesConfig::default(), &files(&root)).unwrap();
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn config_rel_falls_back_without_a_discovered_manifest() {
    let root = fixture("missing-path");
    let findings = check_with_files(&root, &load(&root), &[]).unwrap();
    assert!(
        findings
            .iter()
            .any(|finding| finding.file == ".no-mistakes.yml"),
        "{findings:?}"
    );
}

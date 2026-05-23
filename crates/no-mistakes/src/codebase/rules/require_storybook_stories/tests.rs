use super::*;
use crate::config::v2::schema::{Project, ProjectType, StringOrList};
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/rules/require-storybook-stories")
        .join(name)
}

fn config(options: &str) -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.projects.insert(
        "web".to_string(),
        Project {
            type_: Some(ProjectType::Nextjs),
            root: Some(".".to_string()),
            ..Default::default()
        },
    );
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        projects: vec!["web".to_string()],
        options: serde_yaml::from_str(options).unwrap(),
        ..Default::default()
    });
    config
}

fn config_with_storybook(options: &str) -> NoMistakesConfig {
    let mut config = config(options);
    config.tests.storybook.configs = Some(StringOrList::One(".storybook/main.ts".to_string()));
    config
}

fn config_with_project_root(root: &str, options: &str) -> NoMistakesConfig {
    let mut config = config(options);
    config.projects.get_mut("web").unwrap().root = Some(root.to_string());
    config
}

#[test]
fn direct_and_transitive_story_coverage_passes() {
    let root = fixture("covered");
    let findings = check(
        &root,
        &config(
            r#"
include_all_react_named_exports: true
"#,
        ),
        None,
    )
    .unwrap();

    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn default_story_config_and_test_exclusion_are_used() {
    let root = fixture("defaults");
    let findings = check(
        &root,
        &config_with_storybook(
            r#"
include_all_react_named_exports: true
"#,
        ),
        None,
    )
    .unwrap();

    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn absolute_story_config_patterns_are_project_relative() {
    let root = fixture("defaults");
    let pattern = root.join("stories/**/*.stories.tsx");
    let relative = config::project_relative_pattern(
        &root,
        &root.join(".storybook"),
        &pattern.to_string_lossy(),
    );

    assert_eq!(relative, "stories/**/*.stories.tsx");
}

#[test]
fn reports_selected_component_without_story() {
    let root = fixture("missing");
    let findings = check(
        &root,
        &config(
            r#"
stories: ["stories/**/*.stories.tsx"]
include_all_react_named_exports: true
"#,
        ),
        None,
    )
    .unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "components/Missing.tsx");
    assert_eq!(
        findings[0].target.as_deref(),
        Some("components/Missing.tsx#Missing")
    );
}

#[test]
fn same_file_siblings_are_not_implicitly_covered() {
    let root = fixture("same-file-sibling");
    let findings = check(
        &root,
        &config(
            r#"
stories: ["stories/**/*.stories.tsx"]
include_all_react_named_exports: true
"#,
        ),
        None,
    )
    .unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(
        findings[0].target.as_deref(),
        Some("components/Card.tsx#Sibling")
    );
}

#[test]
fn helper_imports_do_not_count_as_direct_story_coverage() {
    let root = fixture("helper-import");
    let findings = check(
        &root,
        &config(
            r#"
stories: ["stories/**/*.stories.tsx"]
include_all_react_named_exports: true
"#,
        ),
        None,
    )
    .unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(
        findings[0].target.as_deref(),
        Some("components/Hidden.tsx#Hidden")
    );
}

#[test]
fn transitive_coverage_uses_project_relative_keys() {
    let root = fixture("project-root");
    let findings = check(
        &root,
        &config_with_project_root(
            "web",
            r#"
stories: ["stories/**/*.stories.tsx"]
include_all_react_named_exports: true
"#,
        ),
        None,
    )
    .unwrap();

    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn component_and_file_opt_outs_use_no_mistakes_comments() {
    let root = fixture("comments");
    let findings = check(
        &root,
        &config(
            r#"
stories: ["stories/**/*.stories.tsx"]
include_all_react_named_exports: true
"#,
        ),
        None,
    )
    .unwrap();

    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn config_opt_outs_need_reasons_and_existing_targets() {
    let root = fixture("missing");
    let findings = check(
        &root,
        &config(
            r#"
stories: ["stories/**/*.stories.tsx"]
include_all_react_named_exports: true
allow_components:
  "components/Missing.tsx#Missing": ""
  "components/Gone.tsx#Gone": "no longer exists"
allow_files:
  "components/Card.tsx": "covered by story"
  "components/nope/**": "gone"
"#,
        ),
        None,
    )
    .unwrap();

    assert!(findings
        .iter()
        .any(|finding| finding.message.contains("must include a reason")));
    assert!(findings
        .iter()
        .any(|finding| finding.file == "components/nope/**"
            && finding.message.contains("does not match")));
}

#[test]
fn default_export_assignments_are_selected() {
    let root = fixture("default-export");
    let findings = check(
        &root,
        &config(
            r#"
stories: ["stories/**/*.stories.tsx"]
include_all_react_default_exports: true
"#,
        ),
        None,
    )
    .unwrap();

    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn dynamic_import_targets_are_not_required_by_include_all() {
    let root = fixture("dynamic");
    let findings = check(
        &root,
        &config(
            r#"
stories: ["stories/**/*.stories.tsx"]
include_all_react_named_exports: true
"#,
        ),
        None,
    )
    .unwrap();

    assert!(findings.is_empty(), "{findings:#?}");
}

use super::super::*;
use crate::config::v2::{
    schema::{Project, RuleDef, RuleTestTargets, TestProjectPolicy, Tests, VitestConfig},
    NoMistakesConfig,
};

#[test]
fn project_scoped_test_target_rules_intersect_project_and_test_filters() {
    let root = super::fixture("defaults");
    let unit = root.join("web/unit/example.test.mts");
    let integration = root.join("web/integration-tests/example.test.mts");
    let config = NoMistakesConfig {
        projects: std::collections::BTreeMap::from([(
            "web".to_string(),
            Project {
                root: Some("web".to_string()),
                ..Default::default()
            },
        )]),
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            projects: vec!["web".to_string()],
            tests: RuleTestTargets {
                vitest: vec!["integration".to_string()],
                ..Default::default()
            },
            ..Default::default()
        }],
        tests: Tests {
            vitest: VitestConfig {
                projects: std::collections::BTreeMap::from([(
                    "integration".to_string(),
                    TestProjectPolicy {
                        include: vec!["web/integration-tests/**/*.test.mts".to_string()],
                        ..Default::default()
                    },
                )]),
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    };
    let skip = super::super::super::skip_dir_set(&config);
    let target_roots = super::super::super::target_roots(&root, &config, &config.rules[0]);

    let files = candidate_files(
        &root,
        &config,
        &[
            unit,
            integration.clone(),
            root.join("api/integration-tests/example.test.mts"),
        ],
        &skip,
        &target_roots,
        &config.rules[0],
    );

    assert_eq!(files, vec![integration]);
}

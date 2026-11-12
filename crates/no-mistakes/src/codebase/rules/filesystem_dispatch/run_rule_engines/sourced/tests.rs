use super::*;

#[test]
fn sourced_dispatch_reports_config_paths_and_workflow_policies() {
    for (rule, scenario, configuration, names, expected) in [
        (
            CONFIG_PATH_REFERENCES,
            "",
            "missing-literal",
            vec!["config/app.yml"],
            "config/app.yml",
        ),
        (
            GITHUB_ACTIONS_MAX_INLINE_SCRIPT_LINES,
            "boundary",
            "three",
            vec![
                ".github/workflows/scripts.yml",
                ".github/actions/prepare/action.yaml",
            ],
            ".github/workflows/scripts.yml",
        ),
        (
            GITHUB_ACTIONS_LOCAL_ACTION_CHECKOUT,
            "unit",
            "",
            vec![".github/workflows/order.yml"],
            ".github/workflows/order.yml",
        ),
    ] {
        let base = crate::test_support::rule_fixture_root(rule);
        let root = base.join(scenario);
        let config = if configuration.is_empty() {
            let mut config = NoMistakesConfig::default();
            config.rules.push(crate::config::v2::schema::RuleDef {
                rule: rule.into(),
                scope: Some(crate::config::v2::schema::RuleScope::Repository),
                ..Default::default()
            });
            config
        } else {
            let path = base
                .parent()
                .unwrap()
                .join(format!("configs/{configuration}.yml"));
            serde_yaml::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
        };
        let files = names.iter().map(|name| root.join(name)).collect::<Vec<_>>();
        assert!(files.iter().all(|path| path.is_file()), "{files:?}");
        let sources = crate::codebase::rules::source_store_for_files(&files);
        for deferred in [false, true] {
            let findings = run(rule, &root, &config, &files, &sources, deferred)
                .unwrap()
                .unwrap();
            assert!(
                findings.iter().any(|finding| finding.rule == rule
                    && finding.file == expected
                    && finding.line > 0),
                "{rule}: {findings:?}"
            );
            if rule == GITHUB_ACTIONS_LOCAL_ACTION_CHECKOUT {
                assert_eq!(
                    findings
                        .iter()
                        .map(|finding| finding.line)
                        .collect::<Vec<_>>(),
                    [6, 22, 28, 33, 40, 43, 47, 51]
                );
            }
        }
    }
}

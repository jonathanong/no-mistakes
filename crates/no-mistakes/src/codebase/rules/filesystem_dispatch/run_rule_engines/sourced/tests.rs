use super::*;

#[test]
fn sourced_dispatch_reports_structured_config_pins_and_empty_files() {
    for (rule, scenario, configuration, name, lines) in [
        (
            STRUCTURED_CONFIG_POLICY,
            "",
            "../configs/invalid-yaml.yml",
            "invalid.yml",
            vec![1],
        ),
        (
            GITHUB_ACTIONS_PINNED_HASH,
            "fail",
            ".no-mistakes.yml",
            ".github/workflows/ci.yml",
            vec![7, 9, 11, 13],
        ),
        (
            NO_EMPTY_OR_COMMENTS_ONLY_FILES,
            "fail",
            ".no-mistakes.yml",
            "placeholder.ts",
            vec![1],
        ),
    ] {
        let root = crate::test_support::rule_fixture_root(rule).join(scenario);
        let config: NoMistakesConfig =
            serde_yaml::from_str(&std::fs::read_to_string(root.join(configuration)).unwrap())
                .unwrap();
        let files = [root.join(name)];
        assert!(files[0].is_file(), "{files:?}");
        let sources = crate::codebase::rules::source_store_for_files(&files);
        let findings = run(rule, &root, &config, &files, &sources, false)
            .unwrap()
            .unwrap();
        assert!(
            findings
                .iter()
                .all(|finding| finding.rule == rule && finding.file == name),
            "{rule}: {findings:?}"
        );
        assert_eq!(
            findings
                .iter()
                .map(|finding| finding.line)
                .collect::<Vec<_>>(),
            lines,
            "{rule}: {findings:?}"
        );
    }
}

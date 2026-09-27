fn navigation_helper_fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/rules/config-path-references/navigation-helper-names")
            .join(name),
    )
}

fn navigation_helper_findings(name: &str) -> Vec<RuleFinding> {
    let root = navigation_helper_fixture(name);
    let config: NoMistakesConfig = serde_yaml::from_str(
        &std::fs::read_to_string(root.join(".no-mistakes.yml")).unwrap(),
    )
    .unwrap();
    let files: Vec<_> = [
        ".no-mistakes.yml",
        "playwright.config.ts",
        "app/page.tsx",
        "selectors/button.tsx",
    ]
    .into_iter()
    .map(|file| root.join(file))
    .filter(|file| file.is_file())
    .collect();
    let mut findings = check_with_files(&root, &config, &files).unwrap();
    findings.extend(
        crate::codebase::rules::config_path_references::check_with_files(&root, &config, &files)
            .unwrap(),
    );
    findings
}

#[test]
fn navigation_helper_names_are_not_filesystem_paths() {
    // Neither the bare callee nor the dotted callee is backed by a file.
    let findings = navigation_helper_findings("pass");
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn navigation_helper_names_do_not_disable_real_config_path_checks() {
    let findings = navigation_helper_findings("fail");
    assert_eq!(findings.len(), 6, "{findings:?}");
    for rule in [RULE_ID, "config-path-references"] {
        let messages: Vec<_> = findings
            .iter()
            .filter(|finding| finding.rule == rule)
            .map(|finding| finding.message.as_str())
            .collect();
        assert_eq!(messages.len(), 3, "{messages:?}");
        for field in [
            "tests.playwright.configs[0]",
            "tests.playwright.selectorRoots[0]",
            "tests.playwright.frontendRoot",
        ] {
            assert!(messages.iter().any(|message| message.contains(field)), "{messages:?}");
        }
        assert!(messages.iter().all(|message| !message.contains("navigationHelpers")), "{messages:?}");
    }
}

use super::*;

fn project_fixture() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../test-cases/codebase-analysis/test-no-unmocked-dynamic-imports-project-setups",
        ),
    )
}

fn assert_scoped_findings(findings: &[RuleFinding], expected: &[&str]) {
    let files = findings
        .iter()
        .map(|finding| finding.file.as_str())
        .collect::<HashSet<_>>();
    assert_eq!(files, expected.iter().copied().collect(), "{findings:?}");
}

#[test]
fn explicit_project_setups_use_imported_helpers_without_cross_project_leakage() {
    let root = project_fixture();
    for (config_file, expected) in [
        (
            ".no-mistakes.yml",
            vec![
                "web/excluded.test.mts",
                "web/genuine.test.mts",
                "other/uncovered.test.mts",
                "overlap/shared.test.mts",
                "overlap/helper-cut.test.mts",
                "ordered/order.test.mts",
            ],
        ),
        (
            ".no-mistakes-both.yml",
            vec![
                "web/excluded.test.mts",
                "web/genuine.test.mts",
                "overlap/helper-cut.test.mts",
                "ordered/order.test.mts",
            ],
        ),
        (
            ".no-mistakes-selected.yml",
            vec!["web/genuine.test.mts", "overlap/helper-cut.test.mts"],
        ),
        (
            ".no-mistakes-playwright.yml",
            vec!["overlap/shared.test.mts"],
        ),
        (
            ".no-mistakes-runner-ignored.yml",
            vec!["web/covered.test.mts"],
        ),
        (".no-mistakes-app-project.yml", vec!["web/genuine.test.mts"]),
        (".no-mistakes-ordered.yml", vec![]),
        (".no-mistakes-reversed.yml", vec!["ordered/order.test.mts"]),
    ] {
        let config =
            crate::config::v2::load_v2_config(&root, Some(&root.join(config_file))).unwrap();
        let files =
            crate::codebase::ts_source::discover_files(&root, &config.filesystem.skip_directories);
        let facts = crate::codebase::check_facts::collect_check_facts(
            &root,
            files,
            crate::codebase::check_facts::CheckFactPlan {
                imports: true,
                dynamic_imports: true,
                source: true,
                ..Default::default()
            },
        );
        assert_scoped_findings(&check(&root, &config, None).unwrap(), &expected);
        assert_scoped_findings(
            &check_with_facts(&root, &config, None, &facts).unwrap(),
            &expected,
        );
    }
}

#[test]
fn explicit_project_setups_reject_missing_and_escaping_paths() {
    let root = project_fixture();
    for (name, expected) in [
        (
            ".no-mistakes-missing.yml",
            "missing from the analysis file inventory",
        ),
        (
            ".no-mistakes-invalid.yml",
            "invalid repository-relative path",
        ),
        (
            ".no-mistakes-ignored.yml",
            "missing from the analysis file inventory",
        ),
    ] {
        let config = crate::config::v2::load_v2_config(&root, Some(&root.join(name))).unwrap();
        let error = check(&root, &config, None).unwrap_err();
        assert!(error.to_string().contains(expected), "{name}: {error}");
        let files =
            crate::codebase::ts_source::discover_files(&root, &config.filesystem.skip_directories);
        let facts = crate::codebase::check_facts::collect_check_facts(
            &root,
            files,
            crate::codebase::check_facts::CheckFactPlan {
                imports: true,
                dynamic_imports: true,
                source: true,
                ..Default::default()
            },
        );
        let error = check_with_facts(&root, &config, None, &facts).unwrap_err();
        assert!(error.to_string().contains(expected), "{name}: {error}");
    }
}

#[test]
fn imported_setup_helper_fact_failures_are_reported() {
    let root = project_fixture();
    let helper = root.join("test-helpers/web-mocks.mts");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    for (replacement, expected) in [
        (None, "prepared graph facts are missing"),
        (
            Some(crate::codebase::check_facts::CheckFileFacts {
                parse_error: Some("fixture parse error".to_string()),
                ..Default::default()
            }),
            "failed to parse imported setup helper",
        ),
        (
            Some(crate::codebase::check_facts::CheckFileFacts::default()),
            "missing dynamic import facts for imported setup helper",
        ),
    ] {
        let files = crate::codebase::ts_source::discover_files(&root, &[]);
        let mut facts = crate::codebase::check_facts::collect_check_facts(
            &root,
            files,
            crate::codebase::check_facts::CheckFactPlan {
                imports: true,
                dynamic_imports: true,
                source: true,
                ..Default::default()
            },
        );
        facts.ts.remove(&helper);
        let expect_helper_path = replacement.is_some();
        if let Some(replacement) = replacement {
            facts.ts.insert(helper.clone(), replacement.into());
        }
        let error = check_with_facts(&root, &config, None, &facts).unwrap_err();
        assert!(error.to_string().contains(expected), "{error}");
        if expect_helper_path {
            assert!(error.to_string().contains("web-mocks.mts"), "{error}");
        }
    }
}

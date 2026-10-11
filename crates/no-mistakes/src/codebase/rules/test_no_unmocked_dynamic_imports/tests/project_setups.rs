use super::*;

fn project_fixture() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../test-cases/codebase-analysis/test-no-unmocked-dynamic-imports-project-setups",
        ),
    )
}

fn assert_scoped_findings(findings: &[RuleFinding]) {
    let files = findings
        .iter()
        .map(|finding| finding.file.as_str())
        .collect::<HashSet<_>>();
    assert_eq!(
        files,
        HashSet::from([
            "web/excluded.test.mts",
            "web/genuine.test.mts",
            "other/uncovered.test.mts",
        ]),
        "{findings:?}"
    );
}

#[test]
fn explicit_project_setups_use_imported_helpers_without_cross_project_leakage() {
    let root = project_fixture();
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    assert_scoped_findings(&check(&root, &config, None).unwrap());
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
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
    assert_scoped_findings(&check_with_facts(&root, &config, None, &facts).unwrap());
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
    ] {
        let config = crate::config::v2::load_v2_config(&root, Some(&root.join(name))).unwrap();
        let error = check(&root, &config, None).unwrap_err();
        assert!(error.to_string().contains(expected), "{name}: {error}");
    }
}

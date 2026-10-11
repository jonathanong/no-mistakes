use super::*;

#[test]
fn broad_checks_admit_imported_vitest_project_matchers_and_preserve_exclusions() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../test-cases/codebase-analysis/test-no-unmocked-dynamic-imports-runner-executions/recovered-custom",
        ),
    );
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
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
    for findings in [
        check(&root, &config, None).unwrap(),
        check_with_facts(&root, &config, None, &facts).unwrap(),
        crate::codebase::rules::run_check_with_facts(&root, None, None, &facts).unwrap(),
    ] {
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings[0].file, "custom/genuine.case.mts");
        assert_eq!(findings[0].target.as_deref(), Some("src/genuine.mts"));
    }
}

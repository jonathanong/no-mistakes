use super::*;

#[test]
fn aggregate_dynamic_import_resolution_rejects_ignored_targets() {
    let fixture = crate::test_support::materialize_gitignore_fixture("transitive-visibility");
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let shared = crate::codebase::check_facts::collect_check_facts(
        &root,
        files.clone(),
        crate::codebase::check_facts::CheckFactPlan {
            imports: true,
            dynamic_imports: true,
            source: true,
            ..Default::default()
        },
    );
    let config = crate::config::v2::load_v2_config_from_visible(&root, None, &files).unwrap();
    let tsconfig =
        crate::codebase::ts_resolver::resolve_tsconfig_from_visible(None, &root, &files).unwrap();

    let findings = check_with_prepared_facts(&root, &config, &tsconfig, &shared).unwrap();
    let finding = findings
        .iter()
        .find(|finding| finding.file == "tests/ignored-target.test.ts")
        .expect("ignored dynamic import remains an unresolved unmocked import");

    assert_eq!(finding.import.as_deref(), Some("../dynamic/ignored-target"));
    assert_eq!(finding.target, None);
}

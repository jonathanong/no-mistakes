use super::*;

#[test]
fn run_all_uses_package_tsconfigs_for_canonical_graph_rules() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check/monorepo-tsconfig-catalog");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let results = run_all(fixture.path().to_path_buf(), None, None).unwrap();

    assert!(results.rules.iter().any(|finding| {
        finding.rule == no_mistakes::codebase::rules::FORBIDDEN_DEPENDENCIES
            && finding.file == "packages/app/src/api.ts"
            && finding.target.as_deref() == Some("sharp")
    }));
    assert!(results.rules.iter().any(|finding| {
        finding.rule == no_mistakes::codebase::rules::TEST_NO_UNMOCKED_DYNAMIC_IMPORTS
            && finding.file == "packages/lib/src/entry.ts"
            && finding.target.as_deref() == Some("packages/lib/src/lazy.ts")
    }));
}

#[test]
fn aggregate_queue_uses_each_package_alias_when_aliases_conflict() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check/queue-tsconfig-catalog");
    let fixture = crate::test_support::materialize_saved_fixture(&source);

    let results = run_all(fixture.path().to_path_buf(), None, None).unwrap();

    assert!(
        results.queues.is_empty(),
        "each package's @queues alias must resolve to its own producer and worker: {:#?}",
        results.queues
    );
}

#[test]
fn aggregate_next_config_uses_the_importing_packages_path_alias() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check/nextjs-config-path-alias");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let results = run_all(fixture.path().to_path_buf(), None, None).unwrap();
    assert_eq!(results.rules.len(), 1, "{:?}", results.rules);
    let finding = &results.rules[0];
    assert_eq!(finding.file, "packages/web/next-shared.ts");
    assert!(finding.message.contains("redirect destination '/gone'"));
}

#[test]
fn aggregate_next_config_admits_configured_project_tsconfig_without_a_workspace() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check/nextjs-config-path-alias-project");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let results = run_all(fixture.path().to_path_buf(), None, None).unwrap();
    assert_eq!(results.rules.len(), 1, "{:?}", results.rules);
    assert_eq!(results.rules[0].file, "packages/web/next-shared.ts");
    assert!(results.rules[0]
        .message
        .contains("redirect destination '/gone'"));
}

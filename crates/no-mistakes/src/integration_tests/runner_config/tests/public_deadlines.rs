use super::*;
use crate::integration_tests::types::{
    ConfigDeadlineEvidence, DeclaredDeadlineSlot, RunnerDeadlineStatus,
};

#[test]
fn explicit_deadline_demand_reuses_parsed_facts_and_sources_for_public_projection() {
    let fixture =
        crate::test_support::materialize_saved_fixture(&fixture_root("declared-deadlines"));
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());
    let paths = vec![root.join("vitest.config.ts"), root.join("budgets.ts")];
    let sources = Arc::new(crate::codebase::ts_source::SourceStore::new(Arc::new(
        crate::codebase::ts_source::FileInventory::from_paths(&paths),
    )));
    let catalog = Arc::new(crate::codebase::ts_resolver::TsConfigCatalog::from_visible(
        &root,
        std::slice::from_ref(&root),
        &paths,
    ));
    let mut config = NoMistakesConfig::default();
    config.tests.vitest.configs = Some(StringOrList::One("vitest.config.ts".into()));
    let prepared = prepare_runner_configs_with_deadline_evidence(
        &root,
        &config,
        &paths,
        catalog,
        Arc::clone(&sources),
    );
    crate::ast::begin_parse_count_this_thread(&root);
    crate::ast::with_request_parse_cache(|| {
        let parsed = prepared.parse_all().unwrap();
        let facts = crate::codebase::check_facts::CheckFactMap {
            integration_runner_configs: parsed.files,
            ..Default::default()
        };
        let first = prepared.deadline_evidence(&facts);
        assert_eq!(first[0].status, RunnerDeadlineStatus::NotRequested);
        assert_eq!(first[1].status, RunnerDeadlineStatus::Prepared);
        let ConfigDeadlineEvidence::Prepared { projects, .. } = &first[1].configs[0] else {
            panic!("expected projects");
        };
        let inherited = projects
            .iter()
            .find(|project| project.policy_name.as_deref() == Some("inherited"))
            .unwrap();
        assert!(matches!(
            &inherited.case,
            DeclaredDeadlineSlot::Unknown { .. }
        ));
        assert!(matches!(&inherited.fixture, DeclaredDeadlineSlot::Absent));
        for _ in 0..2 {
            assert_eq!(prepared.deadline_evidence(&facts), first);
        }
    });
    let counts = crate::ast::finish_parse_count(&root);
    assert_eq!(counts.get(&paths[0]), Some(&1));
    assert_eq!(counts.get(&paths[1]), Some(&1));
    assert_eq!(sources.physical_read_count(), 2);
}

#[test]
fn requested_empty_configs_and_unprepared_facts_have_distinct_statuses() {
    let root = fixture_root("declared-deadlines");
    let paths = vec![root.join("vitest.config.ts")];
    let sources = Arc::new(crate::codebase::ts_source::SourceStore::new(Arc::new(
        crate::codebase::ts_source::FileInventory::from_paths(&paths),
    )));
    let catalog = Arc::new(crate::codebase::ts_resolver::TsConfigCatalog::from_visible(
        &root,
        std::slice::from_ref(&root),
        &paths,
    ));
    let mut config = NoMistakesConfig::default();
    config.tests.vitest.configs = Some(StringOrList::Many(Vec::new()));
    let empty = prepare_runner_configs_with_deadline_evidence(
        &root,
        &config,
        &paths,
        Arc::clone(&catalog),
        Arc::clone(&sources),
    );
    let evidence = empty.deadline_evidence(&Default::default());
    assert_eq!(evidence[0].status, RunnerDeadlineStatus::NotRequested);
    assert_eq!(evidence[1].status, RunnerDeadlineStatus::Prepared);
    assert!(evidence[1].configs.is_empty());
    config.tests.vitest.configs = Some(StringOrList::One("vitest.config.ts".into()));
    let missing =
        prepare_runner_configs_with_deadline_evidence(&root, &config, &paths, catalog, sources);
    let evidence = missing.deadline_evidence(&Default::default());
    assert_eq!(evidence[1].status, RunnerDeadlineStatus::Failed);
    assert!(matches!(
        evidence[1].configs[0],
        ConfigDeadlineEvidence::Failed { .. }
    ));
}

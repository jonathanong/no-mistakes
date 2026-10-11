use super::*;
use crate::integration_tests::types::DeadlineValue;

#[test]
fn declared_config_facts_reuse_prepared_sources_and_programs() {
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
    config.tests.vitest.configs = Some(StringOrList::One("vitest.config.ts".to_string()));
    let prepared = prepare_runner_configs_with_deadline_evidence(
        &root,
        &config,
        &paths,
        catalog,
        Arc::clone(&sources),
    );
    crate::ast::begin_parse_count_this_thread(&root);
    crate::ast::with_request_parse_cache(|| {
        for _ in 0..2 {
            let parsed = prepared.parse_all().unwrap();
            let projects = parsed.projects_for(&prepared, Framework::Vitest).unwrap();
            let inherited = projects
                .iter()
                .find(|project| project.policy_name.as_deref() == Some("inherited"))
                .unwrap();
            assert_eq!(
                inherited.declared_deadlines.case.as_ref().unwrap().value,
                DeadlineValue::Unknown(
                    crate::integration_tests::types::DeadlineUnknownReason::UnprovedBinding
                )
            );
        }
    });
    let counts = crate::ast::finish_parse_count(&root);
    assert_eq!(counts.get(&paths[0]), Some(&1));
    assert_eq!(counts.get(&paths[1]), Some(&1));
    assert_eq!(sources.physical_read_count(), 2);
}

#[test]
fn declared_json_budgets_preserve_absence_and_have_no_fabricated_spans() {
    let root = fixture_root("declared-deadlines");
    let prepared = prepare_vitest(&root, StringOrList::One("vitest.projects.json".to_string()));
    let parsed = prepared.parse_all().unwrap();
    let projects = parsed.projects_for(&prepared, Framework::Vitest).unwrap();
    let first = &projects[0].declared_deadlines;
    assert_eq!(
        first.case.as_ref().unwrap().value,
        DeadlineValue::Known(1.0)
    );
    assert_eq!(
        first.hook.as_ref().unwrap().value,
        DeadlineValue::Known(30000.0)
    );
    assert!(first.case.as_ref().unwrap().span.is_none());
    assert_eq!(
        projects[1].declared_deadlines.case.as_ref().unwrap().value,
        DeadlineValue::Known(0.0)
    );
    assert_eq!(
        projects[1].declared_deadlines.hook.as_ref().unwrap().value,
        DeadlineValue::Known(30001.0)
    );
    assert!(projects[2].declared_deadlines.case.is_none());
}

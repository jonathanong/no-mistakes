use super::*;
#[test]
fn low_literals_are_parsed_once_across_rule_applications() {
    let root = super::tests::fixture();
    let mut config = super::tests::config("");
    config.effort = Some(crate::config::v2::Effort::Low);
    config.rules[0].name = Some("first".into());
    let mut second = config.rules[0].clone();
    second.name = Some("second".into());
    config.rules.push(second);
    let files = vec![root.join("embedded.ts"), root.join("schema.json")];
    let sources = super::super::source_store_for_files(&files);
    super::super::analysis_policy::prepare_rule_analysis(&root, &mut config, &files, &sources)
        .unwrap();
    let mut plan = crate::codebase::check_facts::CheckFactPlan::default();
    crate::codebase::postgres::configure_prepared_postgres_plan(&root, &config, &mut plan).unwrap();
    crate::codebase::postgres::assign_prepared_embedded_sql_options(&config, &mut plan).unwrap();
    let facts =
        crate::codebase::check_facts::collect_check_facts_with_graph_files_playwright_and_sources(
            &root,
            files.clone(),
            Vec::new(),
            plan,
            None,
            sources.clone(),
        );
    assert!(facts.ts.values().all(|file| file.embedded_sql.is_empty()));
    let observer = crate::diagnostics::InvocationObserver::new(true);
    crate::diagnostics::with_observer(Some(observer.clone()), || {
        check_with_files_sources_and_facts(&root, &config, &files, &sources, &facts).unwrap();
    });
    assert_eq!(
        observer.snapshot().work["postgres.no_transformed_uuid_predicates.low.literal_parses"],
        4
    );
    let reads = sources.physical_read_count();
    check_with_files_sources_and_facts(&root, &config, &files, &sources, &facts).unwrap();
    assert_eq!(sources.physical_read_count(), reads);
}

// Output cases moved to cli_postgres_snapshot_require_query_annotation.
use super::*;

#[test]
fn helper_projection_and_rule_reuse_the_request_sources_and_single_parse() {
    use crate::codebase::ts_source::{FileInventory, SourceStore};
    use std::sync::Arc;
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("helper-tracing-ownership"));
    let mut files = crate::codebase::ts_source::discover_visible_paths(&root);
    let missing = root.join("src/missing.mts");
    files.push(missing.clone());
    let sources = Arc::new(SourceStore::new(Arc::new(FileInventory::from_paths(
        &files,
    ))));
    let config = config_with_options("importSpecifier: './db.mjs'\ninclude: ['src/query.mts']");
    crate::ast::begin_parse_count(&root);
    let facts = crate::codebase::postgres::prepare_rule_sql_facts(
        &root,
        &files,
        Arc::clone(&sources),
        &config,
        &[RULE_ID],
    )
    .unwrap();
    let reads = sources.physical_read_count();
    for _ in 0..2 {
        let _findings =
            check_with_files_sources_and_facts(&root, &config, &files, &sources, &facts).unwrap();
        assert!(facts
            .embedded_sql(&missing, &EmbeddedSqlOptions::configured("./db.mjs", &[]))
            .is_err());
        assert_eq!(sources.physical_read_count(), reads);
    }
    let counts = crate::ast::finish_parse_count(&root);
    assert_eq!(counts.len(), 3, "{counts:?}");
    assert!(counts.values().all(|count| *count == 1), "{counts:?}");
    assert!(!counts.contains_key(&missing));

    // A Playwright demand selects the staged collector, but annotation facts
    // must still come from its single shared parse/read pass.
    let sources = Arc::new(SourceStore::new(Arc::new(FileInventory::from_paths(
        &files,
    ))));
    crate::ast::begin_parse_count(&root);
    let facts =
        crate::codebase::check_facts::collect_check_facts_with_graph_files_playwright_and_sources(
            &root,
            files.clone(),
            Vec::new(),
            crate::codebase::check_facts::CheckFactPlan {
                query_annotation: true,
                embedded_sql: true,
                embedded_sql_options: vec![EmbeddedSqlOptions::configured("./db.mjs", &[])],
                ..Default::default()
            },
            Some(Default::default()),
            Arc::clone(&sources),
        );
    let _findings =
        check_with_files_sources_and_facts(&root, &config, &files, &sources, &facts).unwrap();
    let counts = crate::ast::finish_parse_count(&root);
    assert_eq!(counts.len(), 3, "{counts:?}");
    assert!(counts.values().all(|count| *count == 1), "{counts:?}");
    assert_eq!(sources.physical_read_count(), reads);
}

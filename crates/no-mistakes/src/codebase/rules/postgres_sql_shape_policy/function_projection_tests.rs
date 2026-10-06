use super::function_tests::fixture;
use crate::codebase::check_facts::{
    collect_check_facts_with_graph_files_playwright_and_sources, CheckFactPlan,
};
use std::sync::Arc;

#[test]
fn auxiliary_calls_preserve_legacy_selects_bounds_and_shared_source_identity() {
    let root = fixture();
    let path = root.join("legacy-parity.sql");
    let sql = std::fs::read_to_string(&path).unwrap();
    let direct = crate::codebase::postgres::extract_sql_statement_facts(&sql);
    assert_eq!(direct.selects.len(), 1);
    assert_eq!(direct.bounds.len(), 1);
    assert_eq!(direct.bounds[0].line, 7);
    assert_eq!(
        direct
            .function_calls
            .iter()
            .map(|call| call.line)
            .collect::<Vec<_>>(),
        vec![2, 3, 4]
    );
    for schema in [false, true] {
        let files = vec![path.clone()];
        let sources = crate::codebase::rules::source_store_for_files(&files);
        let facts = collect_check_facts_with_graph_files_playwright_and_sources(
            &root,
            files,
            vec![],
            CheckFactPlan {
                postgres_schema: schema,
                postgres_dml: true,
                postgres_bounds: true,
                postgres_sql_include: vec!["**/*.sql".into()],
                ..Default::default()
            },
            None,
            Arc::clone(&sources),
        );
        let prepared = facts.postgres_statements(&path, None).unwrap();
        assert_eq!(prepared[0].selects, direct.selects);
        assert_eq!(prepared[0].bounds, direct.bounds);
        assert_eq!(prepared[0].function_calls, direct.function_calls);
        assert_eq!(sources.physical_read_count(), 1);
    }
}

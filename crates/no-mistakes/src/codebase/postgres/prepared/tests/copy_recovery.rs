use crate::codebase::check_facts::{
    collect_check_facts_with_graph_files_playwright_and_sources, CheckFactPlan,
};
use std::{path::PathBuf, sync::Arc};

#[test]
fn prepared_sql_recovery_reuses_copy_mask_for_later_table_identity() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-bounded-statements/fixture/prepared-demand"),
    );
    let sql = root.join("copy-recovery.sql");
    let files = vec![sql.clone()];
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let facts = collect_check_facts_with_graph_files_playwright_and_sources(
        &root,
        files,
        vec![],
        CheckFactPlan {
            postgres_dml: true,
            postgres_bounds: true,
            postgres_sql_include: vec!["**/*.sql".into()],
            ..Default::default()
        },
        None,
        Arc::clone(&sources),
    );
    let extracted = facts.postgres_statements(&sql, None).unwrap();
    assert_eq!(extracted.len(), 1);
    assert!(extracted[0].parse_failed);
    assert_eq!(extracted[0].bounds.len(), 1);
    assert_eq!(extracted[0].bounds[0].line, 7);
    let source = sources.read_path(&sql).unwrap();
    let direct = crate::codebase::postgres::extract_sql_statement_facts(&source);
    assert_eq!(extracted[0].bounds, direct.bounds);
    assert_eq!(sources.physical_read_count(), 1);
}

use super::*;

fn fixture_root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-explain-write-rollback/fixture/projection"),
    )
}

fn prepare_scripts(root: &Path, files: &[PathBuf], sources: &SourceStore) -> PreparedPostgresFacts {
    prepare(
        root,
        files,
        sources,
        &CheckFactPlan {
            postgres_explain_write_rollback_paths: Arc::new(files.iter().cloned().collect()),
            ..Default::default()
        },
        &CheckFactMap::default(),
    )
}

#[test]
fn cte_wrapped_direct_writes_retain_outer_locations_and_transaction_outcomes() {
    let root = fixture_root();
    let path = root.join("with-writes.sql");
    let files = std::slice::from_ref(&path);
    let sources = crate::codebase::rules::source_store_for_files(files);
    let facts = prepare_scripts(&root, files, &sources);
    let file = facts.explain_write_rollback(&path).unwrap();
    let plans: Vec<_> = file
        .writes
        .iter()
        .map(|plan| (plan.kind, plan.location.line, plan.autocommit))
        .collect();
    assert_eq!(
        plans,
        [
            ("INSERT", 4, true),
            ("UPDATE", 9, false),
            ("DELETE", 13, false),
            ("MERGE", 17, false),
        ],
    );
    assert!(file.unanalyzable.is_empty());
}

#[test]
fn incomplete_programs_report_unanalyzable_instead_of_passing() {
    let root = fixture_root();
    let partial = root.join("partial-program.sql");
    let invalid = root.join("missing-delimiter.sql");
    let files = [partial.clone(), invalid.clone()];
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let facts = prepare_scripts(&root, &files, &sources);

    let source = sources.read_source_path(&partial).unwrap();
    let statements = crate::codebase::postgres::parse::PreparedSql::new(&source)
        .parse_policy()
        .unwrap();
    assert!(matches!(
        statements.as_slice(),
        [sqlparser::ast::Statement::Explain { .. }]
    ));
    let partial_facts = facts.explain_write_rollback(&partial).unwrap();
    assert!(partial_facts.writes.is_empty());
    assert_eq!(partial_facts.unanalyzable.len(), 1);
    let invalid_facts = facts.explain_write_rollback(&invalid).unwrap();
    assert!(invalid_facts.writes.is_empty());
    assert_eq!(invalid_facts.unanalyzable.len(), 1);
}

#[test]
fn unknown_savepoints_and_dynamic_read_only_settings_report_unanalyzable() {
    let root = fixture_root();
    let controls = root.join("unanalyzable-controls.sql");
    let files = std::slice::from_ref(&controls);
    let sources = crate::codebase::rules::source_store_for_files(files);
    let facts = prepare_scripts(&root, files, &sources);
    let file = facts.explain_write_rollback(&controls).unwrap();
    assert!(file.writes.is_empty());
    let lines: Vec<_> = file
        .unanalyzable
        .iter()
        .map(|item| item.location.line)
        .collect();
    assert_eq!(lines, [2, 3, 6, 7, 8, 9, 10, 11, 12, 13]);
    assert!(format!("{file:?}").contains("UnanalyzablePlan"));
}

#[test]
fn quoted_savepoints_local_set_and_filtered_queries_keep_write_tracking_sound() {
    let root = fixture_root();
    let cases = root.join("coverage-cases.sql");
    let files = std::slice::from_ref(&cases);
    let sources = crate::codebase::rules::source_store_for_files(files);
    let facts = prepare_scripts(&root, files, &sources);
    let file = facts.explain_write_rollback(&cases).unwrap();
    let writes: Vec<_> = file
        .writes
        .iter()
        .map(|plan| (plan.kind, plan.location.line, plan.autocommit))
        .collect();
    assert_eq!(writes, [("UPDATE", 9, false), ("DELETE", 22, true)]);
    assert!(file.unanalyzable.is_empty());
    assert!(format!("{file:?}").contains("WritePlan"));
}

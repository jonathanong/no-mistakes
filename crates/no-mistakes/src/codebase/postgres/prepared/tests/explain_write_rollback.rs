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
    let plans: Vec<_> = facts
        .explain_write_rollback(&path)
        .unwrap()
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
}

#[test]
fn incomplete_programs_do_not_certify_an_explain_location() {
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
    assert!(facts.explain_write_rollback(&partial).unwrap().is_empty());
    assert!(facts.explain_write_rollback(&invalid).unwrap().is_empty());
}

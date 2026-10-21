use super::*;

#[test]
fn files_without_relative_candidates_are_left_untouched() {
    let path = PathBuf::from("/tmp/plain.ts");
    let inventory = Arc::new(FileInventory::from_paths(std::slice::from_ref(&path)));
    let mut files = FileIdMap::with_inventory(Arc::clone(&inventory));
    let call = EmbeddedSqlCall {
        sql_text: Some("SELECT 1".to_string()),
        ..EmbeddedSqlCall::default()
    };
    files.insert(
        path.clone(),
        CheckFileFacts {
            embedded_sql: vec![(
                options(),
                EmbeddedSqlFileFacts {
                    path: path.clone(),
                    executor_bindings: Vec::new(),
                    calls: vec![call],
                    call_spans: vec![(0, 0)],
                    exported_helpers: Default::default(),
                    exported_programs: Default::default(),
                    exported_constants: Default::default(),
                    pending_imported: Vec::new(),
                    imported_call_lines: Default::default(),
                    source_comment_spans: Vec::new(),
                    fragments: Vec::new(),
                    fragment_variants: Vec::new(),
                    fragment_sites: Vec::new(),
                    matched_factory_names: Vec::new(),
                    matched_type_names: Vec::new(),
                    pending_relative: Default::default(),
                    unproven_member_query_spans: Vec::new(),
                    postgres_rule_ids: Vec::new(),
                    record_unproven_spans: true,
                },
            )],
            ..CheckFileFacts::default()
        },
    );
    let sources = SourceStore::new(inventory);
    project_relative_executor_scopes(
        &AnalysisSession::disabled(),
        Path::new("/tmp"),
        &sources,
        None,
        None,
        &mut files,
    );
    let stored = &files.get(&path).expect("file").embedded_sql[0].1;
    assert_eq!(stored.calls.len(), 1);
    assert!(stored.pending_relative.candidates.is_empty());
}

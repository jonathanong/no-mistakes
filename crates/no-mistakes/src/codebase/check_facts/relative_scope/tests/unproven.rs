use super::*;

#[test]
fn unproven_member_queries_without_relative_candidates_use_the_warning_line() {
    let path = PathBuf::from("/tmp/unproven.ts");
    let inventory = Arc::new(FileInventory::from_paths(std::slice::from_ref(&path)));
    let mut files = FileIdMap::with_inventory(Arc::clone(&inventory));
    let other = EmbeddedSqlOptions::configured("@example/db", &["other".into()]);
    let spans = vec![(10, 20), (30, 40)];
    files.insert(
        path.clone(),
        CheckFileFacts {
            embedded_sql: vec![
                (options(), embedded_facts(&path, spans.clone())),
                (other, embedded_facts(&path, vec![(10, 20)])),
            ],
            ..CheckFileFacts::default()
        },
    );
    let sources = SourceStore::new(inventory);
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let count = crate::diagnostics::with_observer(Some(observer.clone()), || {
        project_relative_executor_scopes(
            &AnalysisSession::disabled(),
            Path::new("/tmp"),
            &sources,
            None,
            None,
            &mut files,
        )
    });
    assert_eq!(count, 2, "overlapping profiles count each span once");
    assert_eq!(
        observer
            .snapshot()
            .work
            .get("postgres.unproven_member_query_sql")
            .copied(),
        Some(2)
    );
    assert_eq!(
        unproven_member_query_warning(count).as_deref(),
        Some(
            "warning: 2 member .query calls with SQL-looking text weren't proven to be executors; see docs/migrations/explicit-postgres-executors.md"
        )
    );
    assert!(unproven_member_query_warning(0).is_none());
    let quiet = crate::diagnostics::InvocationObserver::new(false);
    let quiet_count = crate::diagnostics::with_observer(Some(quiet.clone()), || {
        project_relative_executor_scopes(
            &AnalysisSession::disabled(),
            Path::new("/tmp"),
            &sources,
            None,
            None,
            &mut files,
        )
    });
    assert_eq!(quiet_count, 2);
    assert!(!quiet
        .snapshot()
        .work
        .contains_key("postgres.unproven_member_query_sql"));
}

#[test]
fn missing_source_keeps_an_unsuppressed_relative_member() {
    use crate::codebase::postgres::embedded::relative::{
        PendingRelativeCall, PendingRelativeScope, RelativeScopedCandidate,
    };
    let path = PathBuf::from("/tmp/missing-unproven.ts");
    let inventory = Arc::new(FileInventory::from_paths(std::slice::from_ref(&path)));
    let mut files = FileIdMap::with_inventory(Arc::clone(&inventory));
    let mut facts = embedded_facts(&path, Vec::new());
    facts.postgres_rule_ids = vec!["postgres-required-predicates".into()];
    facts.pending_relative = PendingRelativeScope {
        candidates: vec![RelativeScopedCandidate {
            specifier: "./missing".into(),
            imported_name: "openTransaction".into(),
            local_name: "openTransaction".into(),
            factory: true,
            type_import: false,
        }],
        calls: vec![PendingRelativeCall {
            seq: 0,
            owners: vec![0],
            call: EmbeddedSqlCall {
                callee: "query".into(),
                sql_text: Some("SELECT id FROM users".into()),
                ..EmbeddedSqlCall::default()
            },
        }],
        diagnostic_member_sequences: vec![0],
        call_spans: std::collections::BTreeMap::from([(0, (4, 12))]),
        ..PendingRelativeScope::default()
    };
    files.insert(
        path.clone(),
        CheckFileFacts {
            embedded_sql: vec![(options(), facts)],
            ..CheckFileFacts::default()
        },
    );
    let count = project_relative_executor_scopes(
        &AnalysisSession::disabled(),
        Path::new("/tmp"),
        &SourceStore::new(inventory),
        None,
        None,
        &mut files,
    );
    assert_eq!(count, 1, "unreadable source must not suppress the span");
}

use super::{
    or_empty_tsconfig, project_relative_executor_scopes, remember_package_root, workspace_or_empty,
};
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::check_facts::{
    collect_check_facts_with_graph_files_playwright_and_sources,
    collect_check_facts_with_graph_files_playwright_sources_and_session, CheckFactMap,
    CheckFactPlan, CheckFileFacts,
};
use crate::codebase::postgres::{EmbeddedSqlCall, EmbeddedSqlFileFacts, EmbeddedSqlOptions};
use crate::codebase::ts_resolver::{normalize_path, TsConfig};
use crate::codebase::ts_source::{discover_visible_paths, FileIdMap, FileInventory, SourceStore};
use crate::codebase::workspaces::load_indexed_from_source_store;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

fn fixture_root() -> PathBuf {
    normalize_path(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-lock-ordering/fixture/fail-scoped-relative-executors",
    ))
}

fn options() -> EmbeddedSqlOptions {
    EmbeddedSqlOptions::configured("@example/db", &[])
        .with_scoped_executors(&["openTransaction".into()], &["TxExecutor".into()])
}

fn sources_for(root: &Path) -> Arc<SourceStore> {
    Arc::new(SourceStore::new(Arc::new(FileInventory::from_paths(
        &discover_visible_paths(root),
    ))))
}

fn plan() -> CheckFactPlan {
    CheckFactPlan {
        embedded_sql: true,
        embedded_sql_options: vec![options()],
        ..CheckFactPlan::default()
    }
}

fn sql_of(map: &CheckFactMap, path: &Path) -> Vec<String> {
    map.embedded_sql(path, &options())
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()))
        .calls
        .iter()
        .filter_map(|call| call.sql_text.clone())
        .collect()
}

#[test]
fn unseeded_projection_keeps_only_the_configured_package() {
    let root = fixture_root();
    let inside = normalize_path(&root.join("packages/db/src/orders/lock.ts"));
    let outside = normalize_path(&root.join("packages/other/src/orders/lock.ts"));
    let transaction = normalize_path(&root.join("packages/db/src/transaction.ts"));
    let map = collect_check_facts_with_graph_files_playwright_and_sources(
        &root,
        vec![inside.clone(), outside.clone(), transaction.clone()],
        Vec::new(),
        plan(),
        None,
        sources_for(&root),
    );
    let inside_sql = sql_of(&map, &inside);
    assert!(
        inside_sql.iter().any(|sql| sql.contains("FROM orders")),
        "{inside_sql:?}"
    );
    assert!(
        inside_sql.iter().any(|sql| sql.contains("FROM accounts")),
        "{inside_sql:?}"
    );
    let inside_facts = map.embedded_sql(&inside, &options()).expect("inside");
    assert_eq!(inside_facts.matched_factory_names, ["openTransaction"]);
    assert_eq!(inside_facts.matched_type_names, ["TxExecutor"]);
    assert!(inside_facts.pending_relative.candidates.is_empty());
    let outside_facts = map.embedded_sql(&outside, &options()).expect("outside");
    assert!(outside_facts.calls.is_empty(), "{:?}", outside_facts.calls);
    assert!(outside_facts.matched_factory_names.is_empty());
    assert!(outside_facts.matched_type_names.is_empty());
    assert!(map
        .embedded_sql(&transaction, &options())
        .expect("transaction")
        .calls
        .is_empty());
}

#[test]
fn seeded_session_projection_uses_the_request_store() {
    let root = fixture_root();
    let session = AnalysisSession::disabled();
    let sources = session.visible_paths(&root).source_store_for(&root);
    let inside = normalize_path(&root.join("packages/db/src/orders/lock.ts"));
    let outside = normalize_path(&root.join("packages/other/src/orders/lock.ts"));
    let map = collect_check_facts_with_graph_files_playwright_sources_and_session(
        &session,
        &root,
        (vec![inside.clone(), outside.clone()], Vec::new()),
        plan(),
        None,
        sources,
    );
    assert_eq!(sql_of(&map, &inside).len(), 2);
    assert!(sql_of(&map, &outside).is_empty());
}

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
        &mut files,
    );
    let stored = &files.get(&path).expect("file").embedded_sql[0].1;
    assert_eq!(stored.calls.len(), 1);
    assert!(stored.pending_relative.candidates.is_empty());
}

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
        super::unproven_member_query_warning(count).as_deref(),
        Some(
            "warning: 2 member .query calls with SQL-looking text weren't proven to be executors; see docs/migrations/explicit-postgres-executors.md"
        )
    );
    assert!(super::unproven_member_query_warning(0).is_none());
    let quiet = crate::diagnostics::InvocationObserver::new(false);
    let quiet_count = crate::diagnostics::with_observer(Some(quiet.clone()), || {
        project_relative_executor_scopes(
            &AnalysisSession::disabled(),
            Path::new("/tmp"),
            &sources,
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
        &mut files,
    );
    assert_eq!(count, 1, "unreadable source must not suppress the span");
}

fn embedded_facts(path: &Path, spans: Vec<(u32, u32)>) -> EmbeddedSqlFileFacts {
    EmbeddedSqlFileFacts {
        path: path.to_path_buf(),
        executor_bindings: Vec::new(),
        calls: Vec::new(),
        call_spans: Vec::new(),
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
        unproven_member_query_spans: spans,
        postgres_rule_ids: Vec::new(),
        record_unproven_spans: true,
    }
}

#[test]
fn missing_config_falls_back_without_inventing_a_package() {
    let root = PathBuf::from("/repo");
    let config = or_empty_tsconfig(&root, None);
    assert_eq!(config.dir, root);
    assert!(config.paths.is_empty());
    let present = Arc::new(TsConfig {
        dir: root.clone(),
        ..TsConfig::default()
    });
    assert!(Arc::ptr_eq(
        &or_empty_tsconfig(&root, Some(Arc::clone(&present))),
        &present
    ));
    let workspace = workspace_or_empty(Err(anyhow::anyhow!("missing manifest")));
    assert!(workspace.package_by_name("@example/db").is_none());
}

#[test]
fn package_roots_are_cached_only_for_named_workspace_packages() {
    let root = fixture_root();
    let sources = sources_for(&root);
    let workspace = Arc::new(load_indexed_from_source_store(&root, &sources).expect("workspace"));
    let mut cache = HashMap::new();
    let named = workspace
        .package_by_name("@example/db")
        .expect("db")
        .dir
        .clone();
    assert_eq!(
        remember_package_root(&mut cache, "@example/db", &workspace, Some(named.clone()))
            .as_deref(),
        Some(named.as_path())
    );
    assert_eq!(cache.get("@example/db"), Some(&named));
    assert!(
        remember_package_root(&mut cache, "@missing/pkg", &workspace, Some(named.clone()))
            .is_some()
    );
    assert!(!cache.contains_key("@missing/pkg"));
    assert!(remember_package_root(&mut cache, "../transaction", &workspace, Some(named)).is_some());
    assert!(!cache.contains_key("../transaction"));
    assert!(remember_package_root(&mut cache, "@example/db", &workspace, None).is_none());
}

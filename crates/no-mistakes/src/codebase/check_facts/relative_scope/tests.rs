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
                    call_starts: vec![0],
                    fragments: Vec::new(),
                    matched_factory_names: Vec::new(),
                    matched_type_names: Vec::new(),
                    pending_relative: Default::default(),
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

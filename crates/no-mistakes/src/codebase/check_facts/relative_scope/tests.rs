use super::settings::{or_empty_tsconfig, workspace_or_empty};
use super::{
    project_relative_executor_scopes, remember_package_root, unproven_member_query_warning,
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

mod preservation;
mod projection;
mod settings;
mod unproven;

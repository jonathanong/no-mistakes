use super::super::{
    extract_embedded_sql_from_source, EmbeddedSqlCall, EmbeddedSqlFileFacts, EmbeddedSqlOptions,
};
use super::{
    package_name, package_root_for_specifier, project_relative_scoped_facts, PendingRelativeCall,
    PendingRelativeScope, PendingRelativeSpan, RelativeScopedCandidate,
};
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::{discover_visible_paths, FileInventory, SourceStore};
use crate::codebase::workspaces::load_indexed_from_source_store;
use std::path::{Path, PathBuf};
use std::sync::Arc;

mod consumer_escapes;

fn call(sql: &str) -> EmbeddedSqlCall {
    EmbeddedSqlCall {
        sql_text: Some(sql.to_string()),
        callee: "tx".to_string(),
        ..EmbeddedSqlCall::default()
    }
}

fn candidate(
    specifier: &str,
    name: &str,
    factory: bool,
    type_import: bool,
) -> RelativeScopedCandidate {
    RelativeScopedCandidate {
        specifier: specifier.to_string(),
        imported_name: name.to_string(),
        local_name: name.to_string(),
        factory,
        type_import,
    }
}

fn facts(calls: Vec<EmbeddedSqlCall>, pending: PendingRelativeScope) -> EmbeddedSqlFileFacts {
    EmbeddedSqlFileFacts {
        path: PathBuf::from("file.ts"),
        executor_bindings: Vec::new(),
        call_spans: vec![(0, 0); calls.len()],
        exported_helpers: Default::default(),
        exported_programs: Default::default(),
        pending_imported: Vec::new(),
        imported_call_lines: Default::default(),
        source_comment_spans: Vec::new(),
        calls,
        fragments: Vec::new(),
        fragment_variants: Vec::new(),
        fragment_sites: Vec::new(),
        matched_factory_names: Vec::new(),
        matched_type_names: Vec::new(),
        pending_relative: pending,
        unproven_member_query_spans: Vec::new(),
        postgres_rule_ids: Vec::new(),
        record_unproven_spans: true,
    }
}

fn sqls(facts: &EmbeddedSqlFileFacts) -> Vec<String> {
    facts
        .calls
        .iter()
        .filter_map(|call| call.sql_text.clone())
        .collect()
}

fn project(facts: &mut EmbeddedSqlFileFacts, root: Option<&Path>, resolved: Option<PathBuf>) {
    project_relative_scoped_facts(facts, "", root, |_| resolved.clone());
}

fn embedded_fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded")
        .join(name)
}

fn scoped(specifier: &str) -> EmbeddedSqlOptions {
    EmbeddedSqlOptions::configured(specifier, &[])
        .with_scoped_executors(&["openTransaction".into()], &["TxExecutor".into()])
}

fn extract(name: &str, options: &EmbeddedSqlOptions) -> EmbeddedSqlFileFacts {
    let path = embedded_fixture(name);
    let source = std::fs::read_to_string(&path).expect("fixture");
    extract_embedded_sql_from_source(&path, &source, options)
}

fn bodies(facts: &EmbeddedSqlFileFacts) -> Vec<String> {
    facts
        .calls
        .iter()
        .filter_map(|call| call.sql_text.as_deref())
        .map(|sql| sql.trim_start_matches("SELECT id FROM ").to_string())
        .collect()
}

fn fixture_root() -> PathBuf {
    normalize_path(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-lock-ordering/fixture/fail-scoped-relative-executors",
    ))
}

fn workspace(root: &Path) -> crate::codebase::workspaces::IndexedWorkspaceMap {
    let files = discover_visible_paths(root);
    let sources = SourceStore::new(Arc::new(FileInventory::from_paths(&files)));
    load_indexed_from_source_store(root, &sources).expect("workspace")
}

#[test]
fn excluded_profile_drops_relative_member_spans() {
    let pending = PendingRelativeScope {
        candidates: vec![candidate("./tx", "openTransaction", true, false)],
        calls: vec![PendingRelativeCall {
            seq: 1,
            owners: vec![0],
            call: call("SELECT id FROM users"),
        }],
        diagnostic_member_sequences: vec![1],
        call_spans: std::collections::BTreeMap::from([(1, (4, 12))]),
        ..PendingRelativeScope::default()
    };
    let mut kept = facts(Vec::new(), pending.clone());
    project(&mut kept, None, None);
    assert_eq!(kept.unproven_member_query_spans, vec![(4, 12)]);
    let mut dropped = facts(Vec::new(), pending);
    dropped.record_unproven_spans = false;
    project(&mut dropped, None, None);
    assert!(dropped.unproven_member_query_spans.is_empty());
}

mod part1;
mod part2;
mod part3;

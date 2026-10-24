use super::*;
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::dependencies::extract::InvocationKind;
use crate::codebase::dependencies::graph::{DepGraph, GraphBuildPlan, GraphFiles};
use crate::codebase::ts_resolver::TsConfig;
use crate::codebase::ts_source::facts::{collect_ts_facts_with_session_and_context, TsFactContext};

#[test]
fn deferred_local_declarations_and_mixed_callback_facts_keep_one_class_base() {
    use super::super::ancestry::{error_kinds, ErrorKind};
    let root = fixture()
        .parent()
        .unwrap()
        .join("declaration-projection/fixture");
    let source = root.join("src/classes.ts");
    let declaration = root.join("src/base.d.ts");
    let paths = vec![source.clone(), declaration.clone()];
    let files = GraphFiles::from_files_with_resource_candidates_excluding_indexable(
        paths.clone(),
        paths,
        &std::collections::HashSet::from([declaration.clone()]),
    );
    let session = AnalysisSession::new(Some(crate::diagnostics::InvocationObserver::new(true)));
    session.visible_paths(&root);
    let plan = GraphBuildPlan {
        calls: true,
        extends: true,
        ..Default::default()
    };
    let mut fact_plan = plan.ts_fact_plan();
    fact_plan.per_item_calls = true;
    let mut facts = collect_ts_facts_with_session_and_context(
        &session,
        files.indexable(),
        fact_plan,
        &TsFactContext {
            root: root.clone(),
            ..Default::default()
        },
    );
    let file = facts.get(&source).unwrap();
    let id = file
        .ancestry
        .classes
        .iter()
        .find(|class| class.name == "Derived")
        .unwrap()
        .id;
    assert!(file.function_calls.iter().any(|call| call.is_callback
        && call.invocation == InvocationKind::Callback
        && call.callee == "Plain"));
    // This is a prepared-fact compatibility boundary, not a claim that the
    // parser assigns the initializer callback this owner. Reusing a class's
    // owner ID must never make a callback another base or replace its Error base.
    let callback = facts
        .get_mut(&source)
        .unwrap()
        .function_calls
        .iter_mut()
        .find(|call| call.invocation == InvocationKind::Callback && call.callee == "Plain")
        .unwrap();
    callback.caller = Some("Derived".to_string());
    callback.caller_id = Some(id);
    let config = TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        ..Default::default()
    };
    let graph = DepGraph::build_with_plan_files_config_facts_and_session(
        &root,
        &config,
        plan,
        &files,
        None,
        Some(&facts),
        session.clone(),
    )
    .unwrap();
    let classes = graph.class_declarations();
    let index = classes
        .iter()
        .position(|class| class.scope == "Derived")
        .unwrap();
    assert!(error_kinds(&graph, classes)[index] == ErrorKind::Error);
    let work = session.work_snapshot();
    assert_eq!(work.source_reads[&declaration], 1);
    assert_eq!(work.parse_attempts[&declaration], 1);
}

#[test]
fn recovered_primary_class_syntax_cannot_supply_import_ancestry_proof() {
    let root = fixture()
        .parent()
        .unwrap()
        .join("declaration-recovery/fixture");
    let path = root.join("src/classes.ts");
    let plan = GraphBuildPlan {
        extends: true,
        ..Default::default()
    };
    let session = AnalysisSession::disabled();
    let facts = collect_ts_facts_with_session_and_context(
        &session,
        &[path.clone()],
        plan.ts_fact_plan(),
        &TsFactContext {
            root: root.clone(),
            ..Default::default()
        },
    );
    let file = facts.get(&path).unwrap();
    assert!(file.parse_error.is_some());
    assert!(
        !file.ancestry.classes.is_empty(),
        "the saved error must retain its class AST"
    );
    let config = TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        ..Default::default()
    };
    let graph = DepGraph::build_with_plan_files_config_facts_and_session(
        &root,
        &config,
        plan,
        &GraphFiles::from_files(vec![path]),
        None,
        Some(&facts),
        session,
    )
    .unwrap();
    assert!(graph
        .class_declarations()
        .iter()
        .all(|class| class.global_base.is_none()));
}

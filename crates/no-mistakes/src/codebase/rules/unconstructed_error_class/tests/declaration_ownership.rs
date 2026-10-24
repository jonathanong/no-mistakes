use super::*;
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::dependencies::graph::{DepGraph, GraphBuildPlan, GraphFiles, NodeId};
use crate::codebase::ts_resolver::TsConfig;
use crate::codebase::ts_source::facts::{collect_ts_facts_with_session_and_context, TsFactContext};

#[test]
fn ancestry_reuses_sources_and_facts_without_widening_calls_or_imports() {
    let root = fixture().parent().unwrap().join("declarations/fixture");
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let session = AnalysisSession::new(Some(observer));
    session.visible_paths(&root);
    let path = root.join("src/classes.ts");
    let files = GraphFiles::from_files(vec![path.clone()]);
    let plan = GraphBuildPlan {
        imports: true,
        calls: true,
        extends: true,
        ..Default::default()
    };
    let facts = collect_ts_facts_with_session_and_context(
        &session,
        files.indexable(),
        plan.ts_fact_plan(),
        &TsFactContext {
            root: root.clone(),
            ..Default::default()
        },
    );
    let config = TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        ..Default::default()
    };
    let build = |extends| {
        DepGraph::build_with_plan_files_config_facts_and_session(
            &root,
            &config,
            GraphBuildPlan { extends, ..plan },
            &files,
            None,
            Some(&facts),
            session.clone(),
        )
        .unwrap()
    };
    let calls_only = build(false);
    assert!(session
        .work_snapshot()
        .parse_attempts
        .keys()
        .all(|path| !path.to_string_lossy().contains("node_modules")));
    let ancestry = build(true);
    let repeated = build(true);
    assert_eq!(
        calls_only.resolved_call_sites(),
        ancestry.resolved_call_sites()
    );
    assert_eq!(ancestry.class_declarations(), repeated.class_declarations());
    assert_eq!(
        calls_only.dependencies_of_node(&NodeId::file(&path)),
        ancestry.dependencies_of_node(&NodeId::file(&path))
    );
    assert_eq!(files.indexable(), std::slice::from_ref(&path));
    let work = session.work_snapshot();
    for suffix in [
        "src/classes.ts",
        "node_modules/@types/react/index.d.ts",
        "node_modules/cjs-error/index.d.ts",
        "node_modules/external-errors/types/index.d.ts",
        "node_modules/external-errors/types/errors.d.ts",
    ] {
        let file = root.join(suffix);
        assert_eq!(work.source_reads.get(&file), Some(&1), "{suffix}");
        assert_eq!(work.parse_attempts.get(&file), Some(&1), "{suffix}");
    }
    assert_eq!(work.parse_attempts.len(), 5, "{work:#?}");
}

#[test]
fn scoped_project_targets_cannot_fall_through_to_installed_package_namesakes() {
    use super::super::ancestry::{error_kinds, ErrorKind};
    let root = fixture().parent().unwrap().join("declarations/fixture");
    let files = GraphFiles::from_files(vec![root.join("src/classes.ts")]);
    let config = TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        paths: vec![(
            "external-errors".to_string(),
            vec!["hidden/index.ts".to_string()],
        )],
        ..Default::default()
    };
    let graph = DepGraph::build_with_plan_and_files(
        &root,
        &config,
        GraphBuildPlan {
            extends: true,
            ..Default::default()
        },
        &files,
    )
    .unwrap();
    let classes = graph.class_declarations();
    let index = classes
        .iter()
        .position(|class| class.scope == "ImportedError")
        .unwrap();
    assert!(error_kinds(&graph, classes)[index] == ErrorKind::Unknown);
}

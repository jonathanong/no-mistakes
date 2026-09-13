#[test]
fn symbol_runtime_owner_edges_cover_http_process_and_skips() {
    use crate::codebase::analysis_session::PathInterner;
    use crate::codebase::dependencies::extract::FunctionCall;
    use crate::codebase::ts_process_spawn::SpawnEdge;
    use std::collections::HashMap;

    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/codebase-intel/fixture"),
    );
    let path = root.join("packages/api/src/spawn-runner.mts");
    let target = root.join("packages/api/src/spawn-target.mts");
    let route = root.join("packages/api/src/emails.mts");
    let graph_files = GraphFiles::from_parts(
        vec![path.clone(), target.clone(), route.clone()],
        vec![path.clone(), target.clone(), route.clone()],
        [path.clone(), target.clone(), route.clone()],
        vec![],
    );
    let interner = PathInterner::new();
    let call = |callee: &str, arg: Option<&str>| FunctionCall {
        caller: Some("run".to_string()),
        caller_id: None,
        syntactic_caller: Some("run".to_string()),
        callee: callee.to_string(),
        line: 1,
        offset: 0,
        is_callback: false,
        invocation: InvocationKind::Call,
        target_identity: CallTargetIdentity::Unknown,
        callee_binding_scope: None,
        static_arg: arg.map(str::to_string),
        static_cwd: None,
    };
    let mut calls_by_caller = HashMap::new();
    calls_by_caller.insert(
        "run".to_string(),
        vec![
            call("spawn", Some("./spawn-target.mts")),
            call("child_process.exec", Some("node ./spawn-target.mts")),
            call("fetch", Some("/api/users")),
            call("axios.get", Some("/api/users")),
            call("notSpawn", Some("./spawn-target.mts")),
            call("spawn", None),
            call("fetch", None),
        ],
    );
    let http_route_defs = vec![(route.clone(), "/api/users".to_string())];
    let process_spawns = vec![SpawnEdge {
        spawner: path.clone(),
        entry: target.clone(),
    }];
    let mut edges = Vec::new();
    collect_symbol_runtime_owner_file_edges(
        SymbolRuntimeEdgeInputs {
            root: &root,
            path: &path,
            caller_exports: &["run".to_string()],
            caller: "run",
            calls_by_caller: &calls_by_caller,
            http_route_defs: &http_route_defs,
            process_spawns: &process_spawns,
            visible_files: &graph_files,
            interner: &interner,
        },
        &mut edges,
    );
    assert!(
        edges
            .iter()
            .any(|(_, _, kind)| *kind == EdgeKind::HttpCall),
        "{edges:#?}"
    );
    assert!(
        edges
            .iter()
            .any(|(_, _, kind)| *kind == EdgeKind::ProcessSpawn),
        "{edges:#?}"
    );

    let empty_calls = HashMap::new();
    let mut skipped = Vec::new();
    collect_symbol_runtime_owner_file_edges(
        SymbolRuntimeEdgeInputs {
            root: &root,
            path: &path,
            caller_exports: &["run".to_string()],
            caller: "missing",
            calls_by_caller: &empty_calls,
            http_route_defs: &http_route_defs,
            process_spawns: &process_spawns,
            visible_files: &graph_files,
            interner: &interner,
        },
        &mut skipped,
    );
    assert!(skipped.is_empty());

    collect_symbol_runtime_owner_file_edges(
        SymbolRuntimeEdgeInputs {
            root: &root,
            path: &path,
            caller_exports: &["run".to_string()],
            caller: "run",
            calls_by_caller: &calls_by_caller,
            http_route_defs: &[],
            process_spawns: &[],
            visible_files: &graph_files,
            interner: &interner,
        },
        &mut skipped,
    );
    assert!(skipped.is_empty());
}

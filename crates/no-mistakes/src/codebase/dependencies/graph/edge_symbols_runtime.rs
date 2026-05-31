struct SymbolRuntimeEdgeInputs<'a> {
    root: &'a Path,
    path: &'a Path,
    caller_exports: &'a [String],
    caller: &'a str,
    calls_by_caller: &'a HashMap<String, Vec<FunctionCall>>,
    http_route_defs: &'a [(PathBuf, String)],
    process_spawns: &'a [crate::codebase::ts_process_spawn::SpawnEdge],
}

fn collect_symbol_runtime_owner_file_edges(
    inputs: SymbolRuntimeEdgeInputs<'_>,
    edges: &mut Vec<Edge>,
) {
    if inputs.http_route_defs.is_empty() && inputs.process_spawns.is_empty() {
        return;
    }
    let Some(calls) = inputs.calls_by_caller.get(inputs.caller) else {
        return;
    };
    let process_targets =
        symbol_process_targets(inputs.root, inputs.path, calls, inputs.process_spawns);
    let http_targets = symbol_http_targets(inputs.path, calls, inputs.http_route_defs);
    if http_targets.is_empty() && process_targets.is_empty() {
        return;
    }
    for caller_export in inputs.caller_exports {
        let from = NodeId::Symbol {
            file: inputs.path.to_path_buf(),
            symbol: caller_export.clone(),
        };
        for target in &http_targets {
            edges.push((from.clone(), NodeId::File(target.clone()), EdgeKind::HttpCall));
        }
        for target in &process_targets {
            edges.push((
                from.clone(),
                NodeId::File(target.clone()),
                EdgeKind::ProcessSpawn,
            ));
        }
    }
}

fn symbol_process_targets(
    root: &Path,
    path: &Path,
    calls: &[FunctionCall],
    process_spawns: &[crate::codebase::ts_process_spawn::SpawnEdge],
) -> Vec<PathBuf> {
    let mut targets = Vec::new();
    for call in calls {
        if !is_process_spawn_callee(&call.callee) {
            continue;
        }
        let Some(static_arg) = call.static_arg.as_deref() else {
            continue;
        };
        let resolved = if call.callee.ends_with("exec") || call.callee == "exec" {
            crate::codebase::ts_process_spawn::resolve_entry_file_from_shell(
                static_arg,
                call.static_cwd.as_deref(),
                path,
                root,
            )
        } else {
            crate::codebase::ts_process_spawn::resolve_entry_file(
                static_arg,
                call.static_cwd.as_deref(),
                path,
                root,
            )
        };
        if let Some(resolved) = resolved {
            for spawn in process_spawns {
                if spawn.entry == resolved {
                    targets.push(spawn.entry.clone());
                }
            }
        }
    }
    targets.sort();
    targets.dedup();
    targets
}

fn symbol_http_targets(
    path: &Path,
    calls: &[FunctionCall],
    route_defs: &[(PathBuf, String)],
) -> Vec<PathBuf> {
    use crate::codebase::ts_routes::matcher;

    let mut targets = Vec::new();
    for call in calls {
        if !is_http_callee(&call.callee) {
            continue;
        }
        let Some(static_arg) = call.static_arg.as_deref() else {
            continue;
        };
        for (def_file, def_pattern) in route_defs {
            if def_file != path && matcher::matches(static_arg, def_pattern) {
                targets.push(def_file.clone());
            }
        }
    }
    targets.sort();
    targets.dedup();
    targets
}

fn is_http_callee(callee: &str) -> bool {
    const METHODS: &[&str] = &["delete", "get", "head", "options", "patch", "post", "put"];
    callee == "fetch"
        || dotted_callee_parts(callee)
            .is_some_and(|(_, method)| METHODS.contains(&method))
}

fn is_process_spawn_callee(callee: &str) -> bool {
    const FUNCTIONS: &[&str] = &["exec", "execFile", "fork", "spawn"];
    FUNCTIONS.contains(&callee)
        || dotted_callee_parts(callee)
            .is_some_and(|(_, method)| FUNCTIONS.contains(&method))
}

fn dotted_callee_parts(callee: &str) -> Option<(&str, &str)> {
    callee.rsplit_once('.')
}

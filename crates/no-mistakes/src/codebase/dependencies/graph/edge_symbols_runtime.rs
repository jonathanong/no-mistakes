fn collect_symbol_runtime_owner_file_edges(
    path: &Path,
    caller_exports: &[String],
    caller: &str,
    calls_by_caller: &HashMap<String, Vec<FunctionCall>>,
    http_route_defs: &[(PathBuf, String)],
    has_process_spawns: bool,
    edges: &mut Vec<Edge>,
) {
    if http_route_defs.is_empty() && !has_process_spawns {
        return;
    }
    let Some(calls) = calls_by_caller.get(caller) else {
        return;
    };
    let caller_has_process_spawn =
        has_process_spawns && calls.iter().any(|call| is_process_spawn_callee(&call.callee));
    let http_targets = symbol_http_targets(path, calls, http_route_defs);
    if http_targets.is_empty() && !caller_has_process_spawn {
        return;
    }
    for caller_export in caller_exports {
        let from = NodeId::Symbol {
            file: path.to_path_buf(),
            symbol: caller_export.clone(),
        };
        for target in &http_targets {
            edges.push((from.clone(), NodeId::File(target.clone()), EdgeKind::HttpCall));
        }
        if caller_has_process_spawn {
            edges.push((
                from.clone(),
                NodeId::File(path.to_path_buf()),
                EdgeKind::ProcessSpawn,
            ));
        }
    }
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
    const OWNERS: &[&str] = &["axios", "globalThis", "http", "https", "self", "window"];
    callee == "fetch"
        || dotted_callee_parts(callee)
            .is_some_and(|(owner, method)| OWNERS.contains(&owner) && METHODS.contains(&method))
}

fn is_process_spawn_callee(callee: &str) -> bool {
    const FUNCTIONS: &[&str] = &["exec", "execFile", "fork", "spawn"];
    const OWNERS: &[&str] = &["child_process", "cp", "process"];
    FUNCTIONS.contains(&callee)
        || dotted_callee_parts(callee)
            .is_some_and(|(owner, method)| OWNERS.contains(&owner) && FUNCTIONS.contains(&method))
}

fn dotted_callee_parts(callee: &str) -> Option<(&str, &str)> {
    callee.rsplit_once('.')
}

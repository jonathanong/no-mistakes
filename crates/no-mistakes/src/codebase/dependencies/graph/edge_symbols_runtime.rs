fn collect_symbol_runtime_owner_file_edges(
    path: &Path,
    caller_exports: &[String],
    caller: &str,
    calls_by_caller: &HashMap<String, Vec<String>>,
    has_http_calls: bool,
    has_process_spawns: bool,
    edges: &mut Vec<Edge>,
) {
    if !has_http_calls && !has_process_spawns {
        return;
    }
    let Some(calls) = calls_by_caller.get(caller) else {
        return;
    };
    let caller_has_http_call = has_http_calls && calls.iter().any(|callee| is_http_callee(callee));
    let caller_has_process_spawn =
        has_process_spawns && calls.iter().any(|callee| is_process_spawn_callee(callee));
    if !caller_has_http_call && !caller_has_process_spawn {
        return;
    }
    for caller_export in caller_exports {
        let from = NodeId::Symbol {
            file: path.to_path_buf(),
            symbol: caller_export.clone(),
        };
        if caller_has_http_call {
            edges.push((from.clone(), NodeId::File(path.to_path_buf()), EdgeKind::HttpCall));
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

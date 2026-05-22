// ── HTTP call edges ───────────────────────────────────────────────────────────

/// Collect `HttpCall` edges: files that make literal HTTP calls to paths that
/// match a backend route definition.
///
/// Route definitions and backend prefixes must be configured by
/// `http-route-static-paths`, `http-call-static-paths`, or legacy
/// `route-consistency` options.
/// HTTP client calls are any `.<verb>(literal_path)` or `fetch(literal_path)`
/// where `literal_path` starts with a known backend prefix.
///
/// Runs defensively: non-literal call sites produce no edge. The
/// `http-call-static-paths` guardrail enforces literal discipline.
fn collect_http_call_edges(
    root: &Path,
    tsconfig: &TsConfig,
    facts: Option<&TsFactMap>,
    files: &[(PathBuf, String)],
    graph_files: &[PathBuf],
    all_files: &[PathBuf],
    config_options: Option<&GraphConfigOptions>,
) -> Vec<Edge> {
    use crate::codebase::ts_http_calls::extract_http_calls;

    let Some(config_options) = config_options else {
        return vec![];
    };
    let backend_prefixes = resolved_backend_prefixes(config_options);
    if backend_prefixes.is_empty() {
        return vec![];
    }

    // Collect backend route definitions: (file, pattern)
    let mut route_defs = match (
        resolved_backend_pattern(config_options),
        resolved_backend_register_object(config_options),
    ) {
        (Some(backend_pattern), Some(register_object)) => compile_graph_glob(&backend_pattern)
            .map(|gs| {
                collect_backend_routes_from_graph_inputs(
                    root,
                    all_files,
                    &register_object,
                    &gs,
                    facts,
                    config_options.test_filter.as_ref(),
                )
            })
            .unwrap_or_default(),
        _ => Vec::new(),
    };
    route_defs.extend(collect_next_route_handler_defs(root, all_files, config_options));
    if route_defs.is_empty() {
        return vec![];
    }
    let prefix_strs: Vec<&str> = backend_prefixes.iter().map(String::as_str).collect();

    let _ = tsconfig; // reserved for future alias-aware call resolution

    if let Some(facts) = facts {
        return graph_files
            .par_iter()
            .filter_map(|caller| {
                facts
                    .get(caller)
                    .map(|file_facts| (caller.as_path(), file_facts.http_calls.as_slice()))
            })
            .flat_map_iter(|(caller, calls)| http_edges_for_calls(caller, calls, &route_defs))
            .collect();
    }

    // For each source file, find HTTP calls and match against route defs.
    files
        .par_iter()
        .flat_map_iter(|(caller, source)| {
            let calls = extract_http_calls(source, &prefix_strs);
            http_edges_for_calls(caller, &calls, &route_defs)
        })
        .collect()
}

fn collect_next_route_handler_defs(
    root: &Path,
    all_files: &[PathBuf],
    config_options: &GraphConfigOptions,
) -> Vec<(PathBuf, String)> {
    let frontend_root = if !config_options.route.frontend_root.is_empty() {
        root.join(&config_options.route.frontend_root)
    } else {
        return Vec::new();
    };

    all_files
        .par_iter()
        .filter(|path| path.starts_with(&frontend_root))
        .filter(|path| path.file_stem().and_then(|name| name.to_str()) == Some("route"))
        .filter(|path| {
            matches!(
                path.extension().and_then(|ext| ext.to_str()),
                Some("ts" | "tsx" | "js" | "jsx")
            )
        })
        .filter_map(|path| {
            let rel = path.strip_prefix(&frontend_root).ok()?;
            Some((path.clone(), next_route_handler_pattern(rel)))
        })
        .collect()
}

fn next_route_handler_pattern(relative: &Path) -> String {
    let route_like = relative.with_file_name("page.tsx");
    crate::codebase::ts_routes::defs_frontend::path_to_route_pattern(&route_like)
}

fn http_edges_for_calls(
    caller: &Path,
    calls: &[crate::codebase::ts_http_calls::HttpCall],
    route_defs: &[(PathBuf, String)],
) -> Vec<Edge> {
    use crate::codebase::ts_routes::matcher;

    let mut edges = Vec::new();
    for call in calls {
        for (def_file, def_pattern) in route_defs {
            if def_file != caller && matcher::matches(&call.path, def_pattern) {
                edges.push((
                    NodeId::File(caller.to_path_buf()),
                    NodeId::File(def_file.clone()),
                    EdgeKind::HttpCall,
                ));
            }
        }
    }
    edges
}

// ── Process spawn edges ───────────────────────────────────────────────────────

/// Collect `ProcessSpawn` edges from any file that spawns another via
/// `spawn`/`exec`/`execFile`/`fork` or Playwright `webServer.command`.
///
/// String-literal and template-literal (quasis concatenated) commands are
/// resolved; dynamic expressions are silently skipped.
fn collect_process_spawn_edges(
    root: &Path,
    facts: Option<&TsFactMap>,
    files: &[(PathBuf, String)],
    graph_files: &[PathBuf],
) -> Vec<Edge> {
    use crate::codebase::ts_process_spawn::extract_spawn_edges;

    if let Some(facts) = facts {
        return graph_files
            .par_iter()
            .filter_map(|path| facts.get(path))
            .flat_map_iter(|file_facts| {
                file_facts.process_spawns.iter().map(|e| {
                    (
                        NodeId::File(e.spawner.clone()),
                        NodeId::File(e.entry.clone()),
                        EdgeKind::ProcessSpawn,
                    )
                })
            })
            .collect();
    }

    files
        .par_iter()
        .flat_map_iter(|(spawner, source)| {
            extract_spawn_edges(source, spawner, root)
                .into_iter()
                .map(|e| {
                    (
                        NodeId::File(e.spawner),
                        NodeId::File(e.entry),
                        EdgeKind::ProcessSpawn,
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

// ── Filter spec ──────────────────────────────────────────────────────────────

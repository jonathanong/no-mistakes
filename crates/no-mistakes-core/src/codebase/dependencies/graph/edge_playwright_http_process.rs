fn collect_playwright_route_edges(root: &Path, all_files: &[PathBuf]) -> Vec<Edge> {
    let Ok(report) =
        crate::codebase::playwright_coverage::collect_report_from_files(root, None, &[], all_files)
    else {
        return vec![];
    };

    let frontend_root = playwright_frontend_root(root);
    let all_file_set: HashSet<PathBuf> = all_files.iter().cloned().collect();
    let mut edges = Vec::new();
    for route in report.routes {
        let page_file = root.join(&route.file);
        for test in route.tests {
            edges.push((
                NodeId::File(root.join(test.file)),
                NodeId::File(page_file.clone()),
                EdgeKind::RouteTest,
            ));
        }
        for layout_file in collect_layout_chain_files_from_file_set(
            &page_file,
            &frontend_root,
            &all_file_set,
        ) {
            edges.push((
                NodeId::File(page_file.clone()),
                NodeId::File(layout_file),
                EdgeKind::Layout,
            ));
        }
    }
    edges
}

fn playwright_frontend_root(root: &Path) -> PathBuf {
    let config = crate::codebase::config::load_config(root).ok();
    match crate::codebase::playwright_coverage::resolve_frontend_root(None, root, config.as_ref()) {
        Ok(frontend_root) => frontend_root,
        Err(_) => root.join("web/app"),
    }
}

fn collect_layout_chain_files_from_file_set(
    route_file: &Path,
    frontend_root: &Path,
    all_files: &HashSet<PathBuf>,
) -> Vec<PathBuf> {
    let mut layout_files = Vec::new();
    let mut current = route_file.parent();
    while let Some(parent) = current {
        if !parent.starts_with(frontend_root) {
            break;
        }

        for stem in ["layout", "loading", "error", "not-found", "template"] {
            for ext in ["tsx", "ts", "jsx", "js"] {
                let layout_file = parent.join(format!("{stem}.{ext}"));
                if all_files.contains(&layout_file) {
                    layout_files.push(layout_file);
                }
            }
        }

        current = parent.parent();
    }

    layout_files
}

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
    let Some(backend_pattern) = resolved_backend_pattern(config_options) else {
        return vec![];
    };
    let Some(register_object) = resolved_backend_register_object(config_options) else {
        return vec![];
    };
    let backend_prefixes = resolved_backend_prefixes(config_options);
    if backend_prefixes.is_empty() {
        return vec![];
    }

    let Some(gs) = compile_graph_glob(&backend_pattern) else {
        return vec![];
    };

    // Collect backend route definitions: (file, pattern)
    let route_defs = collect_backend_routes_from_graph_inputs(
        root,
        all_files,
        &register_object,
        &gs,
        facts,
        config_options.test_filter.as_ref(),
    );
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

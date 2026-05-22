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

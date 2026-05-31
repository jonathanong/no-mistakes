fn collect_playwright_route_edges(root: &Path, all_files: &[PathBuf]) -> Vec<Edge> {
    let all_file_set: HashSet<PathBuf> = all_files.iter().cloned().collect();
    let Ok(settings) = crate::playwright::config::load_settings(root, None, &[], None) else {
        return Vec::new();
    };
    let frontend_root = root.join(&settings.frontend_root);
    let Ok(analysis) = crate::playwright::analysis::pipeline::analyze_with_policy(
        root,
        &settings,
        crate::playwright::playwright_tests::TestPolicy {
            assert_conditional_tests: false,
            allow_skipped_tests: false,
        },
        crate::playwright::analysis::types::UniqueSelectorPolicy::default(),
    ) else {
        return Vec::new();
    };

    let mut edges = Vec::new();
    for edge in analysis.edges.edges {
        if let crate::playwright::analysis::types::Edge::Route {
            test_file,
            route_file,
            ..
        } = edge
        {
            let page_file = root.join(route_file.as_str());
            edges.push((
                NodeId::File(root.join(test_file.as_str())),
                NodeId::File(page_file.clone()),
                EdgeKind::RouteTest,
            ));
            for layout_file in
                collect_layout_chain_files_from_file_set(&page_file, &frontend_root, &all_file_set)
            {
                edges.push((
                    NodeId::File(page_file.clone()),
                    NodeId::File(layout_file),
                    EdgeKind::Layout,
                ));
            }
        }
    }
    edges.sort();
    edges.dedup();
    edges
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

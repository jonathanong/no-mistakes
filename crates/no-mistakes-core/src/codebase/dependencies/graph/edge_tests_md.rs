fn collect_test_edges(files: &[PathBuf]) -> Vec<Edge> {
    let file_set: HashSet<&PathBuf> = files.iter().collect();

    let test_exts = ["mts", "ts", "tsx", "mjs", "js", "jsx"];
    let test_variants = ["test", "spec"];

    files
        .par_iter()
        .flat_map_iter(|path| {
            let mut edges = Vec::new();
            let stem = match path.file_stem().and_then(|s| s.to_str()) {
                Some(s) => s.to_string(),
                None => return edges,
            };
            let dir = path.parent().unwrap_or(Path::new(""));

            let source_stem = test_variants.iter().find_map(|&v| {
                let suffix = format!(".{v}");
                stem.strip_suffix(&suffix).map(str::to_string)
            });

            if let Some(src_stem) = source_stem {
                for ext in &test_exts {
                    let src_path = dir.join(format!("{src_stem}.{ext}"));
                    if file_set.contains(&src_path) {
                        edges.push((
                            NodeId::File(path.clone()),
                            NodeId::File(src_path),
                            EdgeKind::TestOf,
                        ));
                    }
                }
            }
            edges
        })
        .collect()
}

/// Collect `MarkdownLink` edges from `.md` files to the files they link to.
fn collect_md_edges(all_files: &[PathBuf], graph_files: &GraphFiles) -> Vec<Edge> {
    let md_files: Vec<PathBuf> = all_files
        .iter()
        .filter(|p| matches!(p.extension().and_then(|e| e.to_str()), Some("md" | "mdx")))
        .cloned()
        .collect();

    md_files
        .into_par_iter()
        .flat_map_iter(|path| {
            let source = match std::fs::read_to_string(&path) {
                Ok(s) => s,
                Err(_) => return vec![],
            };
            let dir = path.parent().unwrap_or(Path::new("")).to_path_buf();
            crate::codebase::md_links::extract_links(&source)
                .into_iter()
                .filter_map(|link| {
                    if crate::codebase::md_links::is_external(&link) {
                        return None;
                    }
                    let target = dir.join(&link);
                    let target_str = target.to_string_lossy();
                    let clean = target_str
                        .split('?')
                        .next()
                        .unwrap_or(&target_str)
                        .split('#')
                        .next()
                        .unwrap_or(&target_str);
                    let target = PathBuf::from(clean);
                    // Resolve `..` lexically (no filesystem access) so the path
                    // matches the normalized form used elsewhere in the graph.
                    let target = crate::codebase::ts_resolver::normalize_path(&target);
                    if graph_files.is_visible(&target) {
                        Some((
                            NodeId::File(path.clone()),
                            NodeId::File(target),
                            EdgeKind::MarkdownLink,
                        ))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

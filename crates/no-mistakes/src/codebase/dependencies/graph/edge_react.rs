fn collect_react_render_edges(
    root: &Path,
    facts: Option<&dyn TsFactLookup>,
    files: &[PathBuf],
) -> Vec<Edge> {
    let Some(facts) = facts else {
        return Vec::new();
    };
    let graph_files: HashSet<PathBuf> = files.iter().cloned().collect();

    files
        .par_iter()
        .filter_map(|path| facts.get_ts_facts(path).map(|file_facts| (path, file_facts)))
        .flat_map_iter(|(path, file_facts)| {
            file_facts
                .react_components
                .iter()
                .flat_map(|component| {
                    component.children.iter().filter_map(|child| {
                        let child_path = crate::codebase::ts_resolver::normalize_path(
                            &root.join(&child.file),
                        );
                        if child_path == *path {
                            return None;
                        }
                        if !graph_files.contains(&child_path) {
                            return None;
                        }
                        Some((
                            NodeId::File(path.clone()),
                            NodeId::File(child_path),
                            EdgeKind::ReactRender,
                        ))
                    })
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

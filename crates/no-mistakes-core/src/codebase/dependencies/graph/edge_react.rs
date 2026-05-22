fn collect_react_render_edges(
    root: &Path,
    facts: Option<&TsFactMap>,
    files: &[PathBuf],
) -> Vec<Edge> {
    let Some(facts) = facts else {
        return Vec::new();
    };

    files
        .par_iter()
        .filter_map(|path| facts.get(path).map(|file_facts| (path, file_facts)))
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

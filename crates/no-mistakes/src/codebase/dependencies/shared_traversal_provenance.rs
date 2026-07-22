fn visible_provenance_path(shared: &SharedTraversalContext, config: PathBuf) -> PathBuf {
    let root_config = shared.root.join("tsconfig.json");
    if root_config.canonicalize().ok().as_deref() == Some(config.as_path()) {
        return root_config
            .strip_prefix(&shared.root)
            .unwrap_or(&root_config)
            .to_path_buf();
    }
    let visible = shared.dataset.paths_for(&shared.root);
    let visible_config = visible.iter().find(|visible| {
        visible.canonicalize().ok().as_deref() == Some(config.as_path())
    });
    let config = visible_config.unwrap_or(&config);
    config
        .strip_prefix(&shared.root)
        .unwrap_or(config)
        .to_path_buf()
}

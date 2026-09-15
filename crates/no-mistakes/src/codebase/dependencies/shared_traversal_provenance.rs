fn visible_provenance_path(shared: &SharedTraversalContext, config: PathBuf) -> PathBuf {
    let map = shared.provenance_by_canonical.get_or_init(|| {
        let mut map = HashMap::new();
        let root_config = shared.root.join("tsconfig.json");
        if let Ok(canonical) = root_config.canonicalize() {
            map.insert(
                crate::codebase::ts_resolver::normalize_path(&canonical),
                PathBuf::from("tsconfig.json"),
            );
        }
        for visible in shared.dataset.paths_for(&shared.root).iter() {
            let name = visible.file_name().and_then(|name| name.to_str());
            if !matches!(name, Some("tsconfig.json" | "jsconfig.json")) {
                continue;
            }
            let Ok(canonical) = visible.canonicalize() else {
                continue;
            };
            let relative = visible
                .strip_prefix(&shared.root)
                .unwrap_or(visible)
                .to_path_buf();
            map.entry(crate::codebase::ts_resolver::normalize_path(&canonical))
                .or_insert(relative);
        }
        map
    });
    map.get(&config).cloned().unwrap_or_else(|| {
        config
            .strip_prefix(&shared.root)
            .unwrap_or(&config)
            .to_path_buf()
    })
}

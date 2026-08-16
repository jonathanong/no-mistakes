fn explicit_existing_entry_files(args: &TraverseArgs, root: &Path, cwd: &Path) -> Vec<PathBuf> {
    args.files
        .iter()
        .enumerate()
        .filter_map(|(index, raw)| {
            let structured = args
                .file_entrypoints_are_structured
                .get(index)
                .copied()
                .unwrap_or(false);
            let raw_file = if structured {
                raw.clone()
            } else {
                parse_entrypoint(&raw.to_string_lossy()).0
            };
            let path = if raw_file.is_absolute() {
                raw_file
            } else {
                let from_root = root.join(&raw_file);
                if from_root.exists() {
                    from_root
                } else {
                    cwd.join(raw_file)
                }
            };
            path.is_file()
                .then(|| crate::codebase::ts_resolver::normalize_path(&path))
        })
        .collect()
}

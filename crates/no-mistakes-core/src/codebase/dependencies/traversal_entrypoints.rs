fn raw_looks_like_source_file(raw: &str, path: &Path) -> bool {
    let has_source_extension = Path::new(raw)
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .is_some_and(|extension| {
            crate::codebase::ts_source::TS_JS_EXTENSIONS.contains(&extension)
        });
    if !has_source_extension {
        return false;
    }
    if !raw.contains('/') && !raw.contains('\\') {
        return true;
    }
    path.parent().is_some_and(Path::exists)
}

fn package_dir_entry(
    dir: &Path,
    workspace: &crate::codebase::workspaces::WorkspaceMap,
) -> Option<PathBuf> {
    workspace
        .packages
        .iter()
        .find(|package| package.dir == dir)
        .and_then(|package| package.entry.clone())
        .or_else(|| {
            [
                "src/index.mts",
                "src/index.ts",
                "src/index.tsx",
                "src/index.cts",
                "src/index.js",
                "src/index.mjs",
                "src/index.jsx",
                "src/index.cjs",
                "index.mts",
                "index.ts",
                "index.tsx",
                "index.cts",
                "index.js",
                "index.mjs",
                "index.jsx",
                "index.cjs",
            ]
            .iter()
            .map(|candidate| dir.join(candidate))
            .find(|candidate| candidate.is_file())
        })
}

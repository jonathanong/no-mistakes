use no_mistakes::codebase::config::InferredRoots;
use std::path::{Path, PathBuf};

const NEXT_CONFIGS: &[&str] = &[
    "next.config.js",
    "next.config.mjs",
    "next.config.ts",
    "next.config.mts",
];
const REMIX_CONFIGS: &[&str] = &[
    "remix.config.js",
    "remix.config.ts",
    "remix.config.mjs",
    "remix.config.mts",
    "remix.config.cjs",
    "remix.config.cts",
];
const VITE_CONFIGS: &[&str] = &[
    "vite.config.js",
    "vite.config.ts",
    "vite.config.mjs",
    "vite.config.mts",
    "vite.config.cjs",
    "vite.config.cts",
];

pub(in crate::check_discovery) fn infer_project_roots_from_files(
    root: &Path,
    files: &[PathBuf],
) -> InferredRoots {
    let files: Vec<_> = files
        .iter()
        .filter(|path| !under_skipped_directory(root, path))
        .collect();
    let nextjs = unique_config_root(&files, NEXT_CONFIGS);
    let explicit_remix = unique_config_root(&files, REMIX_CONFIGS);
    let vite_configs: Vec<_> = files
        .iter()
        .filter(|path| has_basename(path, VITE_CONFIGS))
        .map(|path| (path.as_path(), is_remix_vite_config(path)))
        .collect();
    let remix_vite = unique_roots(
        vite_configs
            .iter()
            .filter(|(_, is_remix)| *is_remix)
            .map(|(path, _)| *path),
    );
    let remix = explicit_remix.or(remix_vite);
    let vitejs = unique_roots(
        vite_configs
            .iter()
            .filter(|(_, is_remix)| !*is_remix)
            .map(|(path, _)| *path),
    );
    InferredRoots {
        nextjs: Some(nextjs),
        remix: Some(remix),
        vitejs: Some(vitejs),
    }
}

fn unique_config_root(files: &[&PathBuf], basenames: &[&str]) -> Option<PathBuf> {
    unique_roots(
        files
            .iter()
            .filter(|path| has_basename(path, basenames))
            .map(|path| path.as_path()),
    )
}

fn unique_roots<'a>(paths: impl Iterator<Item = &'a Path>) -> Option<PathBuf> {
    let mut roots: Vec<_> = paths
        .filter_map(|path| path.parent().map(Path::to_path_buf))
        .collect();
    roots.sort();
    roots.dedup();
    match roots.as_slice() {
        [root] => Some(root.clone()),
        _ => None,
    }
}

fn has_basename(path: &Path, basenames: &[&str]) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| basenames.contains(&name))
}

fn under_skipped_directory(root: &Path, path: &Path) -> bool {
    path.strip_prefix(root).ok().is_some_and(|relative| {
        relative.components().any(|component| {
            component
                .as_os_str()
                .to_str()
                .is_some_and(no_mistakes::codebase::ts_source::is_skipped_dir)
        })
    })
}

fn is_remix_vite_config(path: &Path) -> bool {
    std::fs::read_to_string(path)
        .map(|content| {
            content.contains("@remix-run/dev") || content.contains("vitePlugin as remix")
        })
        .unwrap_or(false)
}

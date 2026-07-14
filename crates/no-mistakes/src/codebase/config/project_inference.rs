use std::path::{Path, PathBuf};

const NEXT_CONFIG_NAMES: &[&str] = &[
    "next.config.js",
    "next.config.mjs",
    "next.config.ts",
    "next.config.mts",
];
const REMIX_CONFIG_NAMES: &[&str] = &[
    "remix.config.js",
    "remix.config.ts",
    "remix.config.mjs",
    "remix.config.mts",
    "remix.config.cjs",
    "remix.config.cts",
];
const VITE_CONFIG_NAMES: &[&str] = &[
    "vite.config.js",
    "vite.config.ts",
    "vite.config.mjs",
    "vite.config.mts",
    "vite.config.cjs",
    "vite.config.cts",
];

fn discover_configs(root: &Path, names: &[&str]) -> Vec<PathBuf> {
    crate::codebase::ts_source::discover_with_basenames(root, &[], names)
}

fn infer_framework_root_from_visible(
    root: &Path,
    visible_paths: &[PathBuf],
    config_names: &[&str],
    filter: impl Fn(&Path) -> bool,
) -> Option<PathBuf> {
    let mut roots = visible_paths
        .iter()
        .filter(|path| path.starts_with(root))
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| config_names.contains(&name))
        })
        .filter(|path| filter(path))
        .filter_map(|path| path.parent().map(Path::to_path_buf))
        .collect::<Vec<_>>();
    roots.sort();
    roots.dedup();
    match roots.as_slice() {
        [root] => Some(root.clone()),
        _ => None,
    }
}

pub fn infer_nextjs_root(root: &Path) -> Option<PathBuf> {
    infer_nextjs_root_from_visible(root, &discover_configs(root, NEXT_CONFIG_NAMES))
}

pub fn infer_nextjs_root_from_visible(root: &Path, visible_paths: &[PathBuf]) -> Option<PathBuf> {
    infer_framework_root_from_visible(root, visible_paths, NEXT_CONFIG_NAMES, |_| true)
}

fn is_remix_vite_config(path: &Path) -> bool {
    std::fs::read_to_string(path)
        .map(|content| {
            content.contains("@remix-run/dev") || content.contains("vitePlugin as remix")
        })
        .unwrap_or(false)
}

pub fn infer_remix_root(root: &Path) -> Option<PathBuf> {
    let mut configs = discover_configs(root, REMIX_CONFIG_NAMES);
    configs.extend(discover_configs(root, VITE_CONFIG_NAMES));
    infer_remix_root_from_visible(root, &configs)
}

pub fn infer_remix_root_from_visible(root: &Path, visible_paths: &[PathBuf]) -> Option<PathBuf> {
    let remix_config_root =
        infer_framework_root_from_visible(root, visible_paths, REMIX_CONFIG_NAMES, |_| true);
    if remix_config_root.is_some() {
        return remix_config_root;
    }
    infer_framework_root_from_visible(root, visible_paths, VITE_CONFIG_NAMES, is_remix_vite_config)
}

pub fn infer_vitejs_root(root: &Path) -> Option<PathBuf> {
    infer_vitejs_root_from_visible(root, &discover_configs(root, VITE_CONFIG_NAMES))
}

pub fn infer_vitejs_root_from_visible(root: &Path, visible_paths: &[PathBuf]) -> Option<PathBuf> {
    infer_framework_root_from_visible(root, visible_paths, VITE_CONFIG_NAMES, |path| {
        !is_remix_vite_config(path)
    })
}

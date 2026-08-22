use crate::codebase::config::{Config, InferredRoots};
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::schema::ProjectType;
use std::path::{Path, PathBuf};

const FRAMEWORK_EXPORTS: &[&str] = &[
    "loader",
    "action",
    "clientLoader",
    "clientAction",
    "meta",
    "links",
    "ErrorBoundary",
];

pub(super) fn is_framework_export(name: &str, is_remix_route_module: bool) -> bool {
    is_remix_route_module && FRAMEWORK_EXPORTS.contains(&name)
}

pub(super) fn configured_roots(
    workspace_root: &Path,
    config: &Config,
    inferred: Option<&InferredRoots>,
) -> Vec<PathBuf> {
    let mut cache = inferred.cloned().unwrap_or_default();
    let mut roots = Vec::new();
    for project in config.projects.values() {
        if project.type_ != Some(ProjectType::Remix) {
            continue;
        }
        roots.push(crate::codebase::ts_resolver::normalize_path(
            &project
                .effective_root_with_cache(workspace_root, &mut cache)
                .unwrap_or_else(|| workspace_root.to_path_buf()),
        ));
    }
    roots.sort();
    roots.dedup();
    roots
}

pub(super) fn is_route_module(path: &Path, remix_roots: &[PathBuf]) -> bool {
    remix_roots
        .iter()
        .any(|root| path.starts_with(root) && is_route_rel(&relative_slash_path(root, path)))
}

fn is_route_rel(rel: &str) -> bool {
    let rel = rel.replace('\\', "/");
    if is_app_root(&rel) {
        return true;
    }
    let Some(rest) = rel
        .strip_prefix("app/routes/")
        .or_else(|| rel.strip_prefix("routes/"))
    else {
        return false;
    };
    route_file_stem(rest).is_some()
}

fn is_app_root(rel: &str) -> bool {
    route_file_stem(rel).is_some_and(|stem| stem == "app/root")
}

fn route_file_stem(rel: &str) -> Option<&str> {
    let stem = strip_ts_js_extension(rel)?;
    if stem.ends_with(".server") || stem.ends_with(".client") {
        return None;
    }
    Some(stem)
}

fn strip_ts_js_extension(path: &str) -> Option<&str> {
    for extension in [".tsx", ".ts", ".jsx", ".js", ".mts", ".cts", ".mjs", ".cjs"] {
        if let Some(stem) = path.strip_suffix(extension) {
            return Some(stem);
        }
    }
    None
}

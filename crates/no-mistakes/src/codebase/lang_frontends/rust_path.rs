use crate::codebase::ts_source::SourceStore;
use regex::Regex;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

pub(super) fn cargo_path_deps(sources: &SourceStore, manifest: &Path) -> Vec<PathBuf> {
    let Some(source) = sources.read_path(manifest).ok() else {
        return Vec::new();
    };
    let parent = manifest.parent().unwrap_or(manifest);
    let mut deps = Vec::new();
    for cap in cargo_path_re().captures_iter(&source) {
        let rel = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        if rel.is_empty() {
            continue;
        }
        deps.push(crate::codebase::ts_resolver::normalize_path(
            &parent.join(rel),
        ));
    }
    deps.sort();
    deps.dedup();
    deps
}

pub(super) fn path_attr_mods(source: &str, file: &Path) -> Vec<PathBuf> {
    let parent = file.parent().unwrap_or(file);
    let mut paths = Vec::new();
    for cap in rust_path_attr_re().captures_iter(source) {
        let rel = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        if rel.is_empty() {
            continue;
        }
        paths.push(crate::codebase::ts_resolver::normalize_path(
            &parent.join(rel),
        ));
    }
    paths.sort();
    paths.dedup();
    paths
}

fn cargo_path_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"(?m)path\s*=\s*"([^"]+)""#).expect("cargo path"))
}

fn rust_path_attr_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"(?s)#\[path\s*=\s*"([^"]+)"\]\s*(?:pub(?:\([^)]+\))?\s+)?mod\s+[A-Za-z_]\w*\s*;"#,
        )
        .expect("path attr")
    })
}

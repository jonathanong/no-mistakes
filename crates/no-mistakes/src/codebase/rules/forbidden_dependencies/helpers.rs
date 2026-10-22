use crate::codebase::dependencies::graph::{DepGraph, NodeId};
use crate::codebase::dependencies::{EdgeKind, RelationshipArg};
use anyhow::Result;
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::path::{Path, PathBuf};

pub(super) fn resolve_root_paths(root: &Path, raw: &str, graph: &DepGraph) -> Vec<PathBuf> {
    let p = Path::new(raw);
    let path = if p.is_absolute() {
        p.to_path_buf()
    } else {
        root.join(raw)
    };
    let normalized = crate::codebase::ts_resolver::normalize_path(&path);
    if normalized.is_file() {
        return vec![normalized];
    }
    if !normalized.is_dir() {
        return Vec::new();
    }
    // Reuse the canonical graph; never rediscover or parse directory contents here.
    graph
        .all_files()
        .filter_map(NodeId::as_file)
        .filter(|path| path.starts_with(&normalized))
        .map(Path::to_path_buf)
        .collect()
}

pub(super) fn build_globset(patterns: &[String]) -> Result<Option<GlobSet>> {
    if patterns.is_empty() {
        return Ok(None);
    }
    let mut builder = GlobSetBuilder::new();
    for p in patterns {
        builder.add(Glob::new(p)?);
    }
    Ok(Some(builder.build()?))
}

pub(super) fn edge_kind_str(k: &EdgeKind) -> String {
    k.as_str().to_string()
}

pub(super) fn repro_command(
    root_str: &str,
    target_name: &str,
    node: &NodeId,
    relationships: &[RelationshipArg],
) -> String {
    let target_flag = match node {
        NodeId::Module(_) => format!("--target-module '{}'", target_name.replace('\'', "'\\''")),
        _ => format!("--filter '{}'", target_name.replace('\'', "'\\''")),
    };
    let rel_flags = if relationships.is_empty() {
        " --relationship all".to_string()
    } else {
        relationships
            .iter()
            .map(|r| format!(" --relationship {}", r.as_str()))
            .collect::<String>()
    };
    format!(
        "no-mistakes dependencies '{}' {target_flag}{rel_flags} --format json",
        root_str.replace('\'', "'\\''")
    )
}

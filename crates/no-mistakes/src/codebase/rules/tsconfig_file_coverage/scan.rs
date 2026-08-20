use super::{finding, CompiledOptions, RuleFinding};
use crate::codebase::ts_resolver::{normalize_path, TsConfigCatalog};
use crate::codebase::ts_source::relative_slash_path;
use crate::codebase::workspaces::IndexedWorkspaceMap;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub(super) fn scan(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Vec<RuleFinding> {
    let tsconfigs = tracked_tsconfigs(root, files);
    if tsconfigs.is_empty() {
        return Vec::new();
    }
    let candidates = typescript_candidates(root, files);
    let mut findings =
        super::lists::list_findings(root, opts, &tsconfigs, &candidates, files, sources);
    let auxiliary = opts
        .auxiliary
        .iter()
        .map(|entry| entry.path.as_str())
        .collect::<BTreeSet<_>>();
    let program_configs = tsconfigs
        .iter()
        .filter(|path| !auxiliary.contains(path.as_str()))
        .map(|path| root.join(path))
        .collect::<Vec<_>>();
    let covered = covered_sources(root, &program_configs, files, sources);
    let allowed = opts
        .allow
        .iter()
        .filter(|entry| !entry.reason.trim().is_empty())
        .map(|entry| entry.path.as_str())
        .collect::<BTreeSet<_>>();
    for (path, rel) in candidates {
        if allowed.contains(rel.as_str()) || covered.contains(&normalize_path(&path)) {
            continue;
        }
        findings.push(finding(
            &rel,
            format!(
                "{rel} is not covered by any tsconfig include/files set; add it to a tsconfig or a reasoned allow entry."
            ),
        ));
    }
    findings
}

pub(super) fn is_tsconfig_path(rel: &str) -> bool {
    if under_node_modules(rel) {
        return false;
    }
    let name = rel.rsplit('/').next().unwrap_or_default();
    name == "tsconfig.json"
        || name
            .strip_prefix("tsconfig.")
            .is_some_and(|suffix| !suffix.is_empty() && suffix.ends_with(".json"))
}

pub(super) fn is_typescript_path(rel: &str, path: &Path) -> bool {
    if under_node_modules(rel)
        || path
            .components()
            .any(|component| component.as_os_str() == "node_modules")
    {
        return false;
    }
    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("ts" | "tsx" | "mts" | "cts")
    )
}

fn tracked_tsconfigs(root: &Path, files: &[PathBuf]) -> BTreeSet<String> {
    files
        .iter()
        .filter_map(|path| {
            let rel = relative_slash_path(root, path);
            is_tsconfig_path(&rel).then_some(rel)
        })
        .collect()
}

fn typescript_candidates(root: &Path, files: &[PathBuf]) -> Vec<(PathBuf, String)> {
    files
        .iter()
        .filter_map(|path| {
            let rel = relative_slash_path(root, path);
            is_typescript_path(&rel, path).then(|| (path.clone(), rel))
        })
        .collect()
}

fn covered_sources(
    root: &Path,
    program_configs: &[PathBuf],
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> BTreeSet<PathBuf> {
    if program_configs.is_empty() {
        return BTreeSet::new();
    }
    TsConfigCatalog::project_source_membership(
        root,
        program_configs,
        files,
        sources,
        &IndexedWorkspaceMap::default(),
    )
    .into_values()
    .flatten()
    .collect()
}

fn under_node_modules(rel: &str) -> bool {
    rel.split('/').any(|component| component == "node_modules")
}

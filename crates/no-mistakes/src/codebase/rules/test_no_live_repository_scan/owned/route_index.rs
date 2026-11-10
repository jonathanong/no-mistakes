//! Where an unsafe import or reference sits. The consumed-file set is unchanged.
use crate::codebase::ts_resolver::{normalize_path, ImportResolver};
use crate::codebase::ts_source::module_facts::arguments::PathProvenance;
use crate::codebase::ts_source::module_facts::arguments::RouteKind;
use crate::codebase::ts_source::module_facts::TypeScriptModuleFacts;
use crate::codebase::workspaces::IndexedWorkspaceMap;
use crate::fx::PathSet;
use std::path::{Path, PathBuf};

pub(super) struct RouteHit {
    pub(super) file: PathBuf,
    pub(super) start: u32,
    pub(super) line: u32,
    pub(super) kind: RouteKind,
    pub(super) detail: String,
    target: PathBuf,
}

pub(super) fn note_resolved(
    hits: &mut Vec<RouteHit>,
    file: &Path,
    provenance: &PathProvenance,
    specifier: &str,
    target: &Path,
) {
    for site in &provenance.route_sites {
        if site.detail == specifier && is_import(site.kind) {
            push(
                hits,
                file,
                site.start,
                site.line,
                site.kind,
                &site.detail,
                target,
            );
        }
    }
}

pub(super) fn unknown_line(provenance: &PathProvenance, specifier: &str) -> u32 {
    provenance
        .route_sites
        .iter()
        .filter(|site| site.detail == specifier)
        .map(|site| site.line)
        .min()
        .unwrap_or(1)
}

pub(super) fn note_nondirect(
    hits: &mut Vec<RouteHit>,
    file: &Path,
    provenance: &PathProvenance,
    module: &TypeScriptModuleFacts,
    resolver: &ImportResolver<'_>,
    workspace: &IndexedWorkspaceMap,
    visible: &PathSet,
) {
    for site in &provenance.route_sites {
        if site.kind != RouteKind::NonDirect {
            continue;
        }
        for import in &module.imports {
            let Some(detail) = import
                .bindings
                .iter()
                .find_map(|binding| {
                    (binding.kind == "named" && !binding.type_only && binding.local == site.detail)
                        .then_some(binding.imported.as_str())
                })
                .or_else(|| {
                    import
                        .bindings
                        .iter()
                        .any(|binding| {
                            binding.kind == "named"
                                && !binding.type_only
                                && binding.imported == site.detail
                        })
                        .then_some(site.detail.as_str())
                })
            else {
                continue;
            };
            let class = resolver.classify_import(&import.specifier, file, workspace, visible);
            if let Some(target) = class.preferred_path() {
                push(hits, file, site.start, site.line, site.kind, detail, target);
            }
        }
    }
}

pub(super) fn link(via: &mut Vec<(PathBuf, PathBuf)>, child: &Path, barrel: &Path) {
    let barrels = forms(barrel);
    for child in forms(child) {
        for barrel in &barrels {
            via.push((child.clone(), barrel.clone()));
        }
    }
}

pub(super) fn targeting<'a>(
    hits: &'a [RouteHit],
    via: &[(PathBuf, PathBuf)],
    file: &Path,
) -> Vec<&'a RouteHit> {
    let accepted = reachable_forms(via, file);
    hits.iter()
        .filter(|hit| accepted.iter().any(|path| path == &hit.target))
        .collect()
}

fn reachable_forms(via: &[(PathBuf, PathBuf)], file: &Path) -> Vec<PathBuf> {
    let mut accepted = Vec::new();
    let mut pending = forms(file);
    while let Some(path) = pending.pop() {
        if accepted.iter().any(|seen| seen == &path) {
            continue;
        }
        for (child, barrel) in via {
            if child == &path {
                pending.push(barrel.clone());
            }
        }
        accepted.push(path);
    }
    accepted
}

fn push(
    hits: &mut Vec<RouteHit>,
    file: &Path,
    start: u32,
    line: u32,
    kind: RouteKind,
    detail: &str,
    target: &Path,
) {
    for target in forms(target) {
        hits.push(RouteHit {
            file: file.to_path_buf(),
            start,
            line,
            kind,
            detail: detail.to_string(),
            target,
        });
    }
}

fn forms(path: &Path) -> Vec<PathBuf> {
    let mut forms = vec![normalize_path(path)];
    if let Ok(real) = path.canonicalize() {
        let real = normalize_path(&real);
        if real != forms[0] {
            forms.push(real);
        }
    }
    forms
}

fn is_import(kind: RouteKind) -> bool {
    matches!(
        kind,
        RouteKind::Namespace
            | RouteKind::Default
            | RouteKind::Dynamic
            | RouteKind::Require
            | RouteKind::ViMock
    )
}

#[cfg(test)]
#[path = "route_index/tests.rs"]
mod tests;

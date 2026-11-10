//! Where an unsafe import or reference sits. The consumed-file set is unchanged.
use crate::codebase::dependencies::graph::TsFactLookup;
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
            push(hits, file, site.start, site.kind, &site.detail, target);
        }
    }
}

pub(super) fn note_unknown(
    slot: &mut Option<(PathBuf, u32)>,
    file: &Path,
    provenance: &PathProvenance,
    specifier: &str,
) {
    let start = provenance
        .route_sites
        .iter()
        .filter(|site| site.detail == specifier)
        .map(|site| site.start)
        .min()
        .unwrap_or(0);
    remember(slot, file, start);
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
            let matched = import.bindings.iter().any(|binding| {
                !binding.type_only
                    && binding.kind == "named"
                    && (binding.local == site.detail || binding.imported == site.detail)
            });
            if !matched {
                continue;
            }
            let class = resolver.classify_import(&import.specifier, file, workspace, visible);
            if let Some(target) = class.preferred_path() {
                push(hits, file, site.start, site.kind, &site.detail, target);
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

pub(super) fn earliest<'a>(
    hits: &'a [RouteHit],
    via: &[(PathBuf, PathBuf)],
    file: &Path,
) -> Option<&'a RouteHit> {
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
    hits.iter()
        .filter(|hit| accepted.iter().any(|path| path == &hit.target))
        .min_by(|left, right| (&left.file, left.start).cmp(&(&right.file, right.start)))
}

pub(super) fn setup_hole(facts: &dyn TsFactLookup, files: &[PathBuf]) -> Option<(PathBuf, u32)> {
    let mut best = None;
    for file in files {
        let name = file
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        if !name.contains("vitest.config") && !name.contains("vite.config") {
            continue;
        }
        let Some(source) = facts
            .get_ts_facts(file)
            .and_then(|facts| facts.source.as_deref())
        else {
            continue;
        };
        let Some(index) = source.find("setupFiles") else {
            continue;
        };
        remember(&mut best, file, index as u32);
    }
    best
}

pub(super) fn note_escape(slot: &mut Option<(PathBuf, u32)>, file: &Path, start: u32) {
    remember(slot, file, start);
}

fn remember(slot: &mut Option<(PathBuf, u32)>, file: &Path, start: u32) {
    let next = (file.to_path_buf(), start);
    if slot.as_ref().is_none_or(|prev| next < *prev) {
        *slot = Some(next);
    }
}

fn push(
    hits: &mut Vec<RouteHit>,
    file: &Path,
    start: u32,
    kind: RouteKind,
    detail: &str,
    target: &Path,
) {
    for target in forms(target) {
        hits.push(RouteHit {
            file: file.to_path_buf(),
            start,
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

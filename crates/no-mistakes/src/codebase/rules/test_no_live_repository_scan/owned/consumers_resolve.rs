//! One shared resolver classifies every unsafe specifier in the run.
use super::super::route_index::{self, specifier_line};
use super::Consumed;
use crate::codebase::ts_resolver::{normalize_path, ImportResolution, VisiblePathLookup};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::codebase::ts_source::module_facts::arguments::PathProvenance;
use crate::codebase::workspaces::IndexedWorkspaceMap;
use std::path::{Path, PathBuf};

#[path = "consumer_selections.rs"]
mod selections;

pub(super) struct Gate<'a> {
    pub(super) resolver: &'a dyn ImportResolution,
    pub(super) workspace: &'a IndexedWorkspaceMap,
    pub(super) visible: &'a dyn VisiblePathLookup,
    pub(super) resolver_name: &'static str,
    pub(super) selection: Option<(
        &'a crate::codebase::dependencies::graph::DepGraph,
        &'a dyn crate::codebase::dependencies::graph::TsFactLookup,
    )>,
}

enum Choice {
    Follow(PathBuf),
    Ignore,
    Unread(&'static str),
}

pub(super) fn apply(
    gate: &Gate<'_>,
    consumed: &mut Consumed,
    file: &Path,
    ts: &TsFileFacts,
    provenance: &PathProvenance,
) {
    let imports = ts
        .module_bindings
        .as_deref()
        .map(|module| module.imports.as_slice())
        .unwrap_or(&[]);
    let source = ts.source.as_deref().unwrap_or("");
    for specifier in &provenance.unsafe_modules {
        match choose(gate, file, specifier) {
            Choice::Follow(target) => {
                let selected =
                    selections::collect(gate, consumed, file, ts, provenance, specifier, &target);
                if route_index::note_resolved(
                    &mut consumed.hits,
                    file,
                    provenance,
                    specifier,
                    &target,
                    &selected,
                ) {
                    consumed.insert(&target);
                }
            }
            Choice::Ignore => {}
            Choice::Unread(reason) => {
                let line = specifier_line(provenance, imports, source, specifier);
                trace(specifier, file, line, gate.resolver_name, reason);
                consumed.unknown_files.insert(file.to_path_buf());
                consumed.unknown_sites.push((file.to_path_buf(), line));
            }
        }
    }
}

/// Path facts were dropped. The file stays unproven without a line-1 unread module.
pub(super) fn note_dropped(consumed: &mut Consumed, file: &Path, facts: &TsFileFacts) {
    consumed.insert(file);
    if debug_enabled() {
        trace(
            "(path facts dropped)",
            file,
            1,
            "path-facts",
            &dropped_reason(facts),
        );
    }
}

fn choose(gate: &Gate<'_>, file: &Path, specifier: &str) -> Choice {
    let class = gate
        .resolver
        .classify_import(specifier, file, gate.workspace, gate.visible);
    if let Some(target) = class.preferred_path() {
        return Choice::Follow(target.to_path_buf());
    }
    if class.is_unresolved_external() && bare_external(gate.resolver, specifier, file) {
        return Choice::Ignore;
    }
    after_miss(gate, file, specifier, !class.is_unresolved_external())
}

fn after_miss(gate: &Gate<'_>, file: &Path, specifier: &str, recognized: bool) -> Choice {
    if let Some(target) = readable_relative(file, specifier) {
        return Choice::Follow(target);
    }
    if recognized {
        if let Some(target) =
            gate.workspace
                .resolve_specifier_from_file_visible(specifier, file, &FilesOnDisk)
        {
            if target.is_file() {
                return Choice::Follow(target);
            }
        }
        return Choice::Unread("workspace package recognized but its entry is not a readable file");
    }
    Choice::Unread("specifier did not resolve")
}

fn readable_relative(importer: &Path, specifier: &str) -> Option<PathBuf> {
    if !specifier.starts_with('.') {
        return None;
    }
    let candidate = normalize_path(&importer.parent()?.join(specifier));
    candidate.is_file().then_some(candidate)
}

fn bare_external(resolver: &dyn ImportResolution, specifier: &str, file: &Path) -> bool {
    !specifier.starts_with(['.', '/', '#']) && !resolver.matches_paths_alias(specifier, file)
}

fn dropped_reason(facts: &TsFileFacts) -> String {
    let Some(module) = facts.module_bindings.as_deref() else {
        return "path facts were dropped".to_string();
    };
    let text = module
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect::<Vec<_>>()
        .join("; ");
    if text.is_empty() {
        "path facts were dropped".to_string()
    } else {
        text
    }
}

fn trace(specifier: &str, file: &Path, line: u32, resolver: &str, reason: &str) {
    if debug_enabled() {
        eprintln!(
            "scan-resolve specifier={specifier} at={}:{} resolver={resolver} reason={reason}",
            file.display(),
            line
        );
    }
}

fn debug_enabled() -> bool {
    std::env::var_os("NO_MISTAKES_SCAN_RESOLVE_DEBUG").is_some()
}

/// Files that exist on disk, used only after the canonical resolver misses.
struct FilesOnDisk;

impl VisiblePathLookup for FilesOnDisk {
    fn contains_visible(&self, path: &Path) -> bool {
        path.is_file()
    }

    fn visible_len(&self) -> usize {
        0
    }

    fn visible_cache_key(&self) -> Vec<PathBuf> {
        Vec::new()
    }
}

#[cfg(test)]
#[path = "consumers_resolve/tests.rs"]
mod tests;

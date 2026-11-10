use super::path_provenance;
use crate::codebase::dependencies::graph::TsFactLookup;
use crate::codebase::ts_source::byte_offset_to_line;
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::codebase::ts_source::module_facts::arguments::RouteKind;
use crate::codebase::ts_source::module_facts::{ModuleDiagnostic, TypeScriptModuleFacts};
use crate::fx::{FxHashMap, FxHashSet};
use std::path::{Path, PathBuf};

#[derive(Default)]
pub(super) struct Escapes {
    local: FxHashMap<PathBuf, FxHashSet<String>>,
    refs: Vec<(PathBuf, u32, u32, String)>,
    /// Files whose path facts failed, or that load a module this pass cannot name.
    unknown_sites: Vec<(PathBuf, u32)>,
}

impl Escapes {
    pub(super) fn new(
        facts: &dyn TsFactLookup,
        graph_files: &[PathBuf],
        reachable_files: &FxHashSet<PathBuf>,
    ) -> Self {
        let mut escapes = Self::default();
        for file in graph_files
            .iter()
            .filter(|file| reachable_files.contains(*file))
        {
            let Some(ts) = facts.get_ts_facts(file) else {
                continue;
            };
            match path_provenance(ts) {
                Some(paths) => {
                    if paths.modules_unknown {
                        escapes.mark_unknown(file, paths.unknown_load.unwrap_or(1));
                    }
                    for site in &paths.route_sites {
                        if site.kind == RouteKind::NonDirect {
                            escapes.refs.push((
                                file.clone(),
                                site.start,
                                site.line,
                                site.detail.clone(),
                            ));
                        }
                    }
                    escapes.local.insert(file.clone(), paths.value_refs.clone());
                }
                // A dropped gate is not an unreadable import. A load the collector
                // already marked unknown still is, at that load's line.
                None => {
                    if let Some(line) = dropped_unreadable_load(ts) {
                        escapes.mark_unknown(file, line);
                    }
                }
            }
        }
        escapes
    }

    pub(super) fn unknown_sites(&self) -> &[(PathBuf, u32)] {
        &self.unknown_sites
    }

    fn mark_unknown(&mut self, file: &Path, line: u32) {
        self.unknown_sites.push((file.to_path_buf(), line));
    }

    pub(super) fn contains(&self, file: &Path, name: &str) -> bool {
        let short = name.rsplit('.').next().unwrap_or(name);
        self.local
            .get(file)
            .is_some_and(|names| names.contains(name) || names.contains(short))
    }

    pub(super) fn reference(&self, file: &Path, name: &str) -> Option<(PathBuf, u32, String)> {
        let short = name.rsplit('.').next().unwrap_or(name);
        self.refs
            .iter()
            .filter(|(path, _, _, detail)| path == file && (detail == name || detail == short))
            .min_by_key(|(_, start, _, _)| *start)
            .map(|(path, _, line, detail)| (path.clone(), *line, detail.clone()))
    }
}

/// Provenance the gate rejected can still name a load that cannot be read.
fn dropped_unreadable_load(ts: &TsFileFacts) -> Option<u32> {
    let module = ts.module_bindings.as_deref()?;
    if let Some(provenance) = module.path_provenance.as_deref() {
        if provenance.modules_unknown {
            return Some(provenance.unknown_load.unwrap_or(1));
        }
    }
    unreadable_diagnostic_line(ts, module)
}

fn unreadable_diagnostic_line(ts: &TsFileFacts, module: &TypeScriptModuleFacts) -> Option<u32> {
    let source = ts.source.as_deref();
    let mut found: Option<u32> = None;
    for diagnostic in &module.diagnostics {
        if !is_unreadable_load(diagnostic) {
            continue;
        }
        let line = diagnostic
            .span
            .as_ref()
            .and_then(|span| source.map(|text| byte_offset_to_line(text, span.start as usize)))
            .unwrap_or(1);
        found = Some(match found {
            Some(prior) => prior.min(line),
            None => line,
        });
    }
    found
}

fn is_unreadable_load(diagnostic: &ModuleDiagnostic) -> bool {
    diagnostic.kind == "unsupported"
        && matches!(
            diagnostic.message.as_str(),
            "Non-literal dynamic import specifier"
                | "Shadowed require is not a module loader"
                | "Non-literal or unsupported require specifier"
                | "Indirect require reference is unsupported"
        )
}

#[cfg(test)]
#[path = "escapes/tests.rs"]
mod tests;

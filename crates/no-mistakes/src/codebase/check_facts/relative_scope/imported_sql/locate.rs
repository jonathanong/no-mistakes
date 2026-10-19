//! Which module declares an exported name, following re-exports.
//!
//! Only already-collected import/export facts and the request resolver are
//! read. Anything that cannot be proven fails closed as [`Located::Unknown`]:
//! a cycle, a chain longer than [`MAX_REEXPORT_HOPS`], an unresolved or
//! unscanned module, a namespace re-export, or two different providers for the
//! same name behind `export *`.

use crate::codebase::check_facts::CheckFileFacts;
use crate::codebase::dependencies::extract::ImportedBindingKind;
use crate::codebase::ts_source::FileIdMap;
use crate::fx::FxHashMap;
use std::path::{Path, PathBuf};

/// Maximum number of re-export hops followed from the module a call imports.
pub(crate) const MAX_REEXPORT_HOPS: u8 = 8;

/// Owned decision for one module, so the hop can recurse without cloning
/// `TsFileFacts`. Only the specifier strings the next hop needs are copied.
enum Step {
    Found(String),
    Forward { specifier: String, imported: String },
    Stars(Vec<String>),
    Absent,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Located {
    /// Declared in `.0` under the local name `.1`.
    Found(PathBuf, String),
    /// This module does not provide the name.
    Absent,
    Unknown,
}

pub(super) struct Locator<'a, R> {
    files: &'a FileIdMap<CheckFileFacts>,
    resolve: R,
    targets: FxHashMap<(PathBuf, String), Option<PathBuf>>,
    located: FxHashMap<(PathBuf, String), Located>,
}

impl<'a, R: FnMut(&str, &Path) -> Option<PathBuf>> Locator<'a, R> {
    pub(super) fn new(files: &'a FileIdMap<CheckFileFacts>, resolve: R) -> Self {
        Self {
            files,
            resolve,
            targets: FxHashMap::default(),
            located: FxHashMap::default(),
        }
    }

    /// The module an import specifier names, resolved once per `(importer, specifier)`.
    pub(super) fn target(&mut self, specifier: &str, from: &Path) -> Option<PathBuf> {
        let key = (from.to_path_buf(), specifier.to_string());
        if let Some(known) = self.targets.get(&key) {
            return known.clone();
        }
        let target = (self.resolve)(specifier, from);
        self.targets.insert(key, target.clone());
        target
    }

    /// Declaration of `name` exported by `file`, memoized per `(file, export)`.
    pub(super) fn locate(&mut self, file: &Path, name: &str) -> Located {
        let key = (file.to_path_buf(), name.to_string());
        if let Some(known) = self.located.get(&key) {
            return known.clone();
        }
        if let Some(observer) = crate::diagnostics::current() {
            observer.increment("postgres.embedded.helper_resolutions", 1);
        }
        let outcome = self.walk(file, name, MAX_REEXPORT_HOPS, &mut Vec::new());
        self.located.insert(key, outcome.clone());
        outcome
    }

    fn walk(
        &mut self,
        file: &Path,
        name: &str,
        hops: u8,
        visiting: &mut Vec<(PathBuf, String)>,
    ) -> Located {
        let key = (file.to_path_buf(), name.to_string());
        if visiting.contains(&key) {
            return Located::Unknown;
        }
        if self.files.get(file).is_none() {
            return Located::Unknown;
        }
        visiting.push(key);
        let outcome = self.provider(file, name, hops, visiting);
        visiting.pop();
        outcome
    }

    /// The next hop, copied out of `&FileIdMap` before the recursive borrow.
    fn step(&self, file: &Path, name: &str) -> Step {
        let Some(facts) = self.files.get(file).map(|file| file.ts.as_ref()) else {
            return Step::Unknown;
        };
        if let Some(binding) = facts
            .exported_bindings
            .iter()
            .find(|binding| binding.exported == name)
        {
            if binding.local == "*" {
                return Step::Unknown;
            }
            let forwarded = match &binding.specifier {
                Some(specifier) => Some((specifier.clone(), binding.local.clone())),
                None => facts
                    .imported_bindings
                    .iter()
                    .find(|import| import.local == binding.local)
                    .map(|import| (import.specifier.clone(), import.imported.clone())),
            };
            let Some((specifier, imported)) = forwarded else {
                return Step::Found(binding.local.clone());
            };
            let namespace = facts.imported_bindings.iter().any(|import| {
                import.local == binding.local && import.kind == ImportedBindingKind::Namespace
            });
            return if namespace {
                Step::Unknown
            } else {
                Step::Forward {
                    specifier,
                    imported,
                }
            };
        }
        if name == "default" {
            return Step::Absent;
        }
        Step::Stars(facts.star_reexport_specifiers.clone())
    }

    fn provider(
        &mut self,
        file: &Path,
        name: &str,
        hops: u8,
        visiting: &mut Vec<(PathBuf, String)>,
    ) -> Located {
        match self.step(file, name) {
            Step::Found(local) => Located::Found(file.to_path_buf(), local),
            Step::Absent => Located::Absent,
            Step::Unknown => Located::Unknown,
            Step::Forward {
                specifier,
                imported,
            } => self.forward(&specifier, file, &imported, hops, visiting),
            Step::Stars(specifiers) => {
                let mut found = None;
                for specifier in specifiers {
                    match self.forward(&specifier, file, name, hops, visiting) {
                        Located::Absent => {}
                        Located::Unknown => return Located::Unknown,
                        provider => match &found {
                            Some(existing) if *existing != provider => return Located::Unknown,
                            _ => found = Some(provider),
                        },
                    }
                }
                found.unwrap_or(Located::Absent)
            }
        }
    }

    fn forward(
        &mut self,
        specifier: &str,
        from: &Path,
        name: &str,
        hops: u8,
        visiting: &mut Vec<(PathBuf, String)>,
    ) -> Located {
        let Some(hops) = hops.checked_sub(1) else {
            return Located::Unknown;
        };
        match self.target(specifier, from) {
            Some(target) => self.walk(&target, name, hops, visiting),
            None => Located::Unknown,
        }
    }
}

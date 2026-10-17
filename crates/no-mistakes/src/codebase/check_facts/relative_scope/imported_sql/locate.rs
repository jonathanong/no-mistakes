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
        let Some(facts) = self.files.get(file).map(|file| file.ts.clone()) else {
            return Located::Unknown;
        };
        visiting.push(key);
        let outcome = self.provider(file, name, hops, visiting, &facts);
        visiting.pop();
        outcome
    }

    fn provider(
        &mut self,
        file: &Path,
        name: &str,
        hops: u8,
        visiting: &mut Vec<(PathBuf, String)>,
        facts: &crate::codebase::ts_source::facts::TsFileFacts,
    ) -> Located {
        if let Some(binding) = facts.exported_bindings.iter().find(|b| b.exported == name) {
            if binding.local == "*" {
                return Located::Unknown;
            }
            let forwarded = match &binding.specifier {
                Some(specifier) => Some((specifier.as_str(), binding.local.as_str())),
                None => facts
                    .imported_bindings
                    .iter()
                    .find(|import| import.local == binding.local)
                    .map(|import| (import.specifier.as_str(), import.imported.as_str())),
            };
            let Some((specifier, imported)) = forwarded else {
                return Located::Found(file.to_path_buf(), binding.local.clone());
            };
            let namespace = facts.imported_bindings.iter().any(|import| {
                import.local == binding.local && import.kind == ImportedBindingKind::Namespace
            });
            return if namespace {
                Located::Unknown
            } else {
                self.forward(specifier, file, imported, hops, visiting)
            };
        }
        if name == "default" {
            return Located::Absent;
        }
        let mut found: Option<Located> = None;
        for specifier in &facts.star_reexport_specifiers {
            match self.forward(specifier, file, name, hops, visiting) {
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

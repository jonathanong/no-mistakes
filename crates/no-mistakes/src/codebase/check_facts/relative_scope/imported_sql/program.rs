//! Runs lowered helper programs across modules.
//!
//! A call site's arguments are known only to its own module; a helper may in
//! turn call helpers imported into *its* module. Each nested call resolves
//! through that module's own import bindings, bounded by [`MAX_CALL_DEPTH`]
//! and a cycle check, and memoized per `(root, module, helper, arguments)`.

use super::locate::{Located, Locator};
use super::CheckFileFacts;
use crate::codebase::dependencies::extract::ImportedBindingKind;
use crate::codebase::postgres::{
    Callee, Callees, EmbeddedSqlOptions, ImportedSite, Outcome, RecoveredSql, Ret, SiteArg,
};
use crate::codebase::ts_source::FileIdMap;
use crate::fx::FxHashMap;
use std::path::{Path, PathBuf};
mod frame;
use frame::Frame;

/// Profile index, executor module, helper module, helper, arguments.
type Key = (usize, PathBuf, PathBuf, String, Vec<SiteArg>);

/// State shared by every frame of one site's execution.
pub(super) struct Shared<'a, R> {
    pub(super) files: &'a FileIdMap<CheckFileFacts>,
    locator: Locator<'a, R>,
    cache: FxHashMap<Key, Option<Vec<Outcome>>>,
}

impl<'a, R: FnMut(&str, &Path) -> Option<PathBuf>> Shared<'a, R> {
    pub(super) fn new(files: &'a FileIdMap<CheckFileFacts>, locator: Locator<'a, R>) -> Self {
        Self {
            files,
            locator,
            cache: FxHashMap::default(),
        }
    }

    /// Declaring module and local name for a helper or selected namespace constant.
    pub(super) fn find(
        &mut self,
        from: &Path,
        name: &str,
        member: Option<&str>,
    ) -> Option<(PathBuf, String)> {
        let file = self.files.get(from)?;
        let import = file
            .ts
            .imported_bindings
            .iter()
            .find(|binding| binding.local == name && !binding.is_type_only);
        let Some(import) = import else {
            return Some((from.to_path_buf(), name.to_string()));
        };
        let exported = match member {
            Some(member) if import.kind == ImportedBindingKind::Namespace => member,
            Some(_) => return None,
            None => &import.imported,
        };
        let target = self.locator.target(&import.specifier, from)?;
        match self.locator.locate(&target, exported) {
            Located::Found(origin, local) => Some((origin, local)),
            _ => None,
        }
    }

    /// The fixed-shape summary of an imported helper, which fills a hole with
    /// exactly one text. A helper in the executor's own module has none.
    fn summary_text(
        &mut self,
        options: &EmbeddedSqlOptions,
        from: &Path,
        site: &ImportedSite,
    ) -> Option<RecoveredSql> {
        let (origin, local) = self.find(from, &site.local, None)?;
        if origin == from {
            return None;
        }
        let (_, embedded) = self
            .files
            .get(&origin)?
            .embedded_sql
            .iter()
            .find(|(known, _)| known == options)?;
        embedded
            .exported_helpers
            .get(&local)?
            .fill(site, &origin, from)
    }

    /// Every alternative text the helper called at `site` contributes.
    pub(super) fn fill(
        &mut self,
        profile: usize,
        options: &EmbeddedSqlOptions,
        from: &Path,
        site: &ImportedSite,
    ) -> Option<Vec<RecoveredSql>> {
        if site.constant.is_some() {
            return super::constants::recovered(self, options, from, site).map(|text| vec![text]);
        }
        if let Some(text) = self.summary_text(options, from, site) {
            return Some(vec![text]);
        }
        let mut frame = Frame::new(self, profile, options, from);
        let outcomes = match frame.call(&site.local, &site.args) {
            Callee::Done(outcomes) => outcomes,
            Callee::Unresolved | Callee::Failed => return None,
        };
        let builder = site.args.iter().position(|arg| *arg == SiteArg::Builder);
        let mut texts = Vec::new();
        for outcome in outcomes {
            let text = hole_text(outcome, builder, site.value_used)?;
            if !texts.contains(&text) {
                texts.push(text);
            }
        }
        (!texts.is_empty()).then_some(texts)
    }
}

/// What fills the hole: the suffix appended to the caller's builder, or the
/// value the helper returned.
fn hole_text(outcome: Outcome, builder: Option<usize>, value_used: bool) -> Option<RecoveredSql> {
    match builder {
        Some(index) => {
            // A value-position call yields the helper's result, which must be
            // the very builder it was given for `base + suffix` to hold.
            if value_used && outcome.ret != Ret::Param(index) {
                return None;
            }
            outcome
                .suffixes
                .into_iter()
                .find(|(param, _)| *param == index)
                .map(|(_, text)| text)
        }
        None => match outcome.ret {
            Ret::Sql(text) => Some(text),
            Ret::Str(_, Some(text)) => Some(text),
            _ => None,
        },
    }
}

#[cfg(test)]
#[path = "program_tests.rs"]
mod program_tests;

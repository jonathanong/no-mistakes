use super::*;
use crate::codebase::dependencies::extract::ImportKind;
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::FxHashMap;
use std::collections::BTreeSet;

pub(super) type PreparedTargets = FxHashMap<PathBuf, FxHashMap<String, ResolvedTarget>>;

#[derive(Clone)]
pub(super) enum ResolvedTarget {
    Primary(Option<PathBuf>),
    Workspace(PathBuf),
}

impl ResolvedTarget {
    fn path(&self) -> Option<&PathBuf> {
        match self {
            Self::Primary(target) => target.as_ref(),
            Self::Workspace(target) => Some(target),
        }
    }
}

/// Traverse the same prepared runtime dependencies consumed by annotation's
/// module index. No discovery, reads, parsing, or mutable evaluator state.
pub(super) fn prepare(
    fallback: &WorkspaceFallback<'_>,
    files: &FileIdMap<CheckFileFacts>,
    seeds: &PathSet,
    primary: &impl Fn(&str, &Path) -> Option<PathBuf>,
) -> (PathSet, PreparedTargets) {
    if fallback.observer.is_some() {
        expand::<true>(fallback, files, seeds, primary)
    } else {
        expand::<false>(fallback, files, seeds, primary)
    }
}

fn expand<const OBSERVED: bool>(
    fallback: &WorkspaceFallback<'_>,
    files: &FileIdMap<CheckFileFacts>,
    seeds: &PathSet,
    primary: &impl Fn(&str, &Path) -> Option<PathBuf>,
) -> (PathSet, PreparedTargets) {
    let mut allowed = seeds.clone();
    let mut pending = seeds.iter().cloned().collect::<BTreeSet<_>>();
    let mut prepared = PreparedTargets::default();
    note::<OBSERVED>(
        fallback,
        "postgres.annotation.closure_seeds",
        seeds.len() as u64,
    );
    while let Some(from) = pending.pop_first() {
        let Some(file) = files.get(&from) else {
            continue;
        };
        let mut targets = FxHashMap::default();
        for specifier in specifiers(&file.ts) {
            let target = fallback.raw_target(specifier, &from, primary);
            if let Some(path) = target.path() {
                if allowed.insert(path.clone()) {
                    pending.insert(path.clone());
                }
            }
            note::<OBSERVED>(fallback, "postgres.annotation.closure_resolutions", 1);
            if OBSERVED && matches!(target, ResolvedTarget::Workspace(_)) {
                note::<OBSERVED>(fallback, "postgres.annotation.closure_workspace_edges", 1);
            }
            targets.insert(specifier.to_owned(), target);
        }
        prepared.insert(from, targets);
    }
    note::<OBSERVED>(
        fallback,
        "postgres.annotation.closure_modules",
        prepared.len() as u64,
    );
    note::<OBSERVED>(
        fallback,
        "postgres.annotation.closure_importers",
        allowed.len() as u64,
    );
    (allowed, prepared)
}

fn note<const OBSERVED: bool>(fallback: &WorkspaceFallback<'_>, metric: &'static str, amount: u64) {
    if OBSERVED {
        fallback
            .observer
            .as_ref()
            .unwrap()
            .increment(metric, amount);
    }
}

fn specifiers(facts: &TsFileFacts) -> BTreeSet<&str> {
    facts
        .imported_bindings
        .iter()
        .filter(|binding| !binding.is_type_only)
        .map(|binding| binding.specifier.as_str())
        .chain(
            facts
                .imports
                .iter()
                .filter(|import| import.kind != ImportKind::Type && !import.computed)
                .map(|import| import.specifier.as_str()),
        )
        .chain(facts.eager_require_specifiers.iter().map(String::as_str))
        .chain(
            facts
                .exported_bindings
                .iter()
                .filter_map(|binding| binding.specifier.as_deref()),
        )
        .chain(facts.star_reexport_specifiers.iter().map(String::as_str))
        .collect()
}

#[cfg(test)]
mod tests;

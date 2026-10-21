use super::CheckFileFacts;
use crate::codebase::postgres::package_root_for_specifier;
use crate::codebase::ts_source::FileIdMap;
use crate::codebase::workspaces::IndexedWorkspaceMap;
use crate::diagnostics::InvocationObserver;
use crate::fx::PathSet;
use std::borrow::Cow;
use std::path::{Path, PathBuf};

mod closure;
use closure::{PreparedTargets, ResolvedTarget};
use std::sync::Arc;

/// Annotation's workspace compatibility fallback obeys the configured rule scope.
/// Ordinary resolver hits keep their existing precedence and visible identity.
pub(super) struct WorkspaceFallback<'a> {
    workspace: &'a IndexedWorkspaceMap,
    visible: &'a PathSet,
    importers: Option<Cow<'a, PathSet>>,
    prepared: PreparedTargets,
    executors: Vec<String>,
    observer: Option<Arc<InvocationObserver>>,
}

impl<'a> WorkspaceFallback<'a> {
    pub(super) fn new(
        workspace: &'a IndexedWorkspaceMap,
        visible: &'a PathSet,
        importers: Option<&'a PathSet>,
        files: &FileIdMap<CheckFileFacts>,
    ) -> Self {
        let mut executors = files
            .values()
            .flat_map(|file| {
                file.query_annotation
                    .iter()
                    .map(|(options, _)| options.import_specifier.clone())
            })
            .filter(|specifier| !specifier.is_empty())
            .collect::<Vec<_>>();
        executors.sort_unstable();
        executors.dedup();
        Self {
            workspace,
            visible,
            importers: importers.map(Cow::Borrowed),
            prepared: PreparedTargets::default(),
            executors,
            observer: crate::diagnostics::current().filter(|observer| observer.detailed()),
        }
    }

    pub(super) fn resolve(
        &self,
        specifier: &str,
        from: &Path,
        primary: impl Fn(&str, &Path) -> Option<PathBuf>,
    ) -> Option<PathBuf> {
        let resolved = self
            .prepared
            .get(from)
            .and_then(|targets| targets.get(specifier))
            .cloned()
            .unwrap_or_else(|| self.raw_target(specifier, from, &primary));
        let target = match resolved {
            ResolvedTarget::Primary(target) => return target,
            ResolvedTarget::Workspace(target) => target,
        };
        let allowed = self
            .importers
            .as_deref()
            .is_none_or(|paths| paths.contains(from))
            || self.executor_target(&target, from, &primary);
        if let Some(observer) = &self.observer {
            observer.increment(
                if allowed {
                    "postgres.annotation.workspace_fallback_edges"
                } else {
                    "postgres.annotation.workspace_fallback_blocked"
                },
                1,
            );
        }
        allowed.then_some(target)
    }

    /// Complete permissions before parallel evaluation; resolver call order
    /// cannot decide whether a transitive helper's imports are followed.
    pub(super) fn prepare(
        &mut self,
        files: &FileIdMap<CheckFileFacts>,
        primary: impl Fn(&str, &Path) -> Option<PathBuf>,
    ) {
        let Some(seeds) = self.importers.as_deref() else {
            return;
        };
        let (importers, targets) =
            crate::perf_trace::trace("postgres.annotation.workspace_closure", || {
                closure::prepare(self, files, seeds, &primary)
            });
        self.importers = Some(Cow::Owned(importers));
        self.prepared = targets;
    }

    fn raw_target(
        &self,
        specifier: &str,
        from: &Path,
        primary: &impl Fn(&str, &Path) -> Option<PathBuf>,
    ) -> ResolvedTarget {
        // Invisible primary hits veto replacement by a workspace target.
        if let Some(target) = primary(specifier, from) {
            return ResolvedTarget::Primary(self.visible.contains(&target).then_some(target));
        }
        self.workspace
            .resolve_specifier_from_file_visible(specifier, from, self.visible)
            .map_or(ResolvedTarget::Primary(None), ResolvedTarget::Workspace)
    }

    fn executor_target(
        &self,
        target: &Path,
        from: &Path,
        primary: &impl Fn(&str, &Path) -> Option<PathBuf>,
    ) -> bool {
        let package = target
            .ancestors()
            .skip(1)
            .find_map(|dir| self.workspace.package_by_dir(dir));
        package.is_some_and(|package| {
            self.executors.iter().any(|specifier| {
                // Alias roots depend on the importing file's catalog. Do not cache
                // a resolution from one package and reuse it for another package.
                package_root_for_specifier(specifier, from, self.workspace, |spec, file| {
                    primary(spec, file).or_else(|| {
                        self.workspace
                            .resolve_specifier_from_file_visible(spec, file, self.visible)
                    })
                })
                .is_some_and(|root| root == package.dir)
            })
        })
    }
}

#[cfg(all(test, feature = "test-instrumentation"))]
mod public_tests;
#[cfg(test)]
mod tests;

#[cfg(all(test, feature = "test-instrumentation"))]
mod transitive_tests;

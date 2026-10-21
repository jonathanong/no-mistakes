use super::CheckFileFacts;
use crate::codebase::postgres::package_root_for_specifier;
use crate::codebase::ts_source::FileIdMap;
use crate::codebase::workspaces::IndexedWorkspaceMap;
use crate::diagnostics::InvocationObserver;
use crate::fx::PathSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Annotation's workspace compatibility fallback obeys the configured rule scope.
/// Ordinary resolver hits keep their existing precedence and visible identity.
pub(super) struct WorkspaceFallback<'a> {
    workspace: &'a IndexedWorkspaceMap,
    visible: &'a PathSet,
    importers: Option<&'a PathSet>,
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
            importers,
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
        // A primary hit outside the inventory must not be replaced with a
        // different visible workspace target for the same import identity.
        if let Some(target) = primary(specifier, from) {
            return self.visible.contains(&target).then_some(target);
        }
        let target =
            self.workspace
                .resolve_specifier_from_file_visible(specifier, from, self.visible)?;
        let allowed = self.importers.is_none_or(|paths| paths.contains(from))
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

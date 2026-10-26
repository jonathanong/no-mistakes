use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::ts_resolver::{ScopedImportResolver, TsConfigCatalog};
use crate::codebase::ts_source::SourceStore;
use crate::config::v2::NoMistakesConfig;
use std::path::{Path, PathBuf};

/// Borrowed ordinary project projection, prepared by the request boundary.
#[doc(hidden)]
pub struct PreparedConfigResolver<'a> {
    resolver: ScopedImportResolver<'a>,
}

impl<'a> PreparedConfigResolver<'a> {
    #[doc(hidden)]
    pub fn new(
        catalog: &'a TsConfigCatalog,
        files: &[PathBuf],
        session: &'a AnalysisSession,
    ) -> Self {
        let visible: crate::fx::PathSet = files.iter().cloned().collect();
        Self {
            resolver: ScopedImportResolver::new_in_session(catalog, &visible, session),
        }
    }

    pub(super) fn resolve_alias(&self, specifier: &str, from: &Path) -> Option<Option<PathBuf>> {
        self.resolver.resolve_path_alias(specifier, from)
    }
}

/// Compatibility entrypoints own missing preparation; engines only borrow it.
pub(crate) fn with_standalone<T>(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &SourceStore,
    run: impl FnOnce(&PreparedConfigResolver<'_>) -> anyhow::Result<T>,
) -> anyhow::Result<T> {
    let mut roots = vec![root.to_path_buf()];
    roots.extend(super::configured_project_roots(root, config));
    roots.sort();
    roots.dedup();
    // Match the ordinary session/catalog's tolerant workspace projection;
    // unrelated malformed manifests must not block config extraction.
    let workspace = crate::codebase::workspaces::load_indexed_from_source_store(root, sources)
        .unwrap_or_default();
    let catalog = TsConfigCatalog::from_visible_and_sources_with_workspace(
        root, &roots, files, sources, &workspace,
    );
    let session = AnalysisSession::new(crate::diagnostics::current());
    let resolver = PreparedConfigResolver::new(&catalog, files, &session);
    run(&resolver)
}

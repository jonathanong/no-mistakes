use crate::codebase::dependencies::extract::is_tsx_file;
use crate::codebase::ts_resolver::{normalize_path, TsConfig};
use crate::codebase::ts_symbols::{extract_symbols_at_path, FileSymbols};
use anyhow::{Context, Result};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

/// Resolved root, tsconfig, and the single absolute target file. Shared setup
/// for every lightweight query command so `--root`/`--tsconfig` fallback and
/// path normalization behave identically (and match `SymbolIndex` keys, which
/// are built from `normalize_path(root.join(rel))`).
pub(crate) struct Target {
    pub root: PathBuf,
    pub visible_paths: Arc<crate::codebase::ts_source::VisiblePathSnapshot>,
    pub abs_file: PathBuf,
    pub visible_files: HashSet<PathBuf>,
    pub session: Arc<crate::codebase::analysis_session::AnalysisSession>,
    pub sources: Arc<crate::codebase::ts_source::SourceStore>,
    dataset: Arc<crate::codebase::analysis_dataset::AnalysisDataset>,
    explicit_tsconfig: Option<PathBuf>,
    tsconfig: OnceLock<std::result::Result<TsConfig, String>>,
}

/// Repository-wide inputs that only reverse queries require. Keeping these out
/// of [`Target`] lets single-file queries stay single-file queries.
pub(crate) struct ReversePrepared {
    pub graph_files: crate::codebase::dependencies::graph::GraphFiles,
    pub tsconfig_catalog: crate::codebase::ts_resolver::TsConfigCatalog,
    pub workspace: Arc<crate::codebase::workspaces::IndexedWorkspaceMap>,
}

pub(crate) fn resolve_target(
    file: &Path,
    root: Option<&Path>,
    tsconfig: Option<&Path>,
) -> Result<Target> {
    let cwd = std::env::current_dir().context("reading current directory")?;
    let root = normalize_path(&crate::cli::resolve_root(
        root.unwrap_or_else(|| Path::new(".")),
        &cwd,
    ));
    let session =
        crate::codebase::analysis_session::AnalysisSession::new(crate::diagnostics::current());
    let dataset = session.dataset(&root);
    let snapshot = dataset.visible_paths_arc();
    let visible_paths = dataset.paths_for(&root);
    let sources = dataset.sources_for(&root);
    let abs_file = resolve_input_file(file, &root, &cwd);
    // Reject a missing target or a directory up front so a typo or stale path
    // is an explicit error rather than an empty (and misleading) result.
    anyhow::ensure!(abs_file.is_file(), "not a file: {}", file.display());
    let explicit_tsconfig = tsconfig.map(|path| {
        normalize_path(&if path.is_absolute() {
            path.to_path_buf()
        } else {
            root.join(path)
        })
    });
    let visible_files = visible_paths
        .iter()
        .map(|path| normalize_path(path))
        .collect();
    Ok(Target {
        root,
        visible_paths: snapshot,
        abs_file,
        visible_files,
        session,
        sources,
        dataset,
        explicit_tsconfig,
        tsconfig: OnceLock::new(),
    })
}

impl Target {
    pub(crate) fn config(
        &self,
        path: Option<&Path>,
    ) -> Result<Arc<crate::config::v2::NoMistakesConfig>> {
        self.dataset.config(path)
    }

    /// Resolve the one configuration a single-file query needs. Reverse
    /// indexes use their ordinary root/workspace catalog, while target-facing
    /// re-export rendering uses this nearest visible config for parity with
    /// `exports-of --no-importers`.
    pub(crate) fn tsconfig(&self) -> Result<&TsConfig> {
        let result = self.tsconfig.get_or_init(|| match self.explicit_tsconfig.as_deref() {
            Some(path) => match self.dataset.tsconfig(Some(path)) {
                Ok(config) => Ok((*config).clone()),
                Err(error) => Err(format!("{error:#}")),
            },
            None => {
                let config = match crate::codebase::ts_resolver::resolve_tsconfig_from_visible_and_sources(
                    None,
                    &self.abs_file,
                    &self.dataset.paths_for(&self.root),
                    &self.sources,
                ) {
                    Ok(config) => config,
                    // Automatic config discovery remains best effort for these
                    // queries. Explicit `--tsconfig` stays authoritative above.
                    Err(_) => TsConfig {
                        dir: self.root.clone(),
                        paths: Vec::new(),
                        paths_dir: self.root.clone(),
                        base_url: None,
                    },
                };
                Ok(config)
            }
        });
        match result {
            Ok(config) => Ok(config),
            Err(error) => Err(anyhow::anyhow!(error.clone())),
        }
    }

    pub(crate) fn prepare_reverse(&self) -> Result<ReversePrepared> {
        let visible_paths = self.dataset.paths_for(&self.root);
        let all = crate::codebase::ts_source::discover_files_from_visible(
            &self.root,
            &[],
            &visible_paths,
        );
        let mut graph_files =
            crate::codebase::dependencies::graph::GraphFiles::from_files_with_resource_candidates(
                all,
                self.visible_paths
                    .tracked_paths_for(&self.root)
                    .as_ref()
                    .clone(),
            );
        graph_files.add_explicit_root(&self.abs_file);
        let workspace = self.dataset.workspace();
        let tsconfig_catalog = match &self.explicit_tsconfig {
            Some(path) => crate::codebase::ts_resolver::TsConfigCatalog::forced(
                &self.root,
                self.tsconfig()?.clone(),
                Some(path.clone()),
            ),
            None => crate::codebase::ts_resolver::TsConfigCatalog::from_visible_and_sources_with_workspace(
                &self.root,
                std::slice::from_ref(&self.root),
                &visible_paths,
                &self.sources,
                &workspace,
            ),
        };
        Ok(ReversePrepared {
            graph_files,
            tsconfig_catalog,
            workspace,
        })
    }
}

/// Resolve the target file against `--root` first, falling back to cwd, then
/// normalize it lexically so it matches discovered/resolved paths.
fn resolve_input_file(file: &Path, root: &Path, cwd: &Path) -> PathBuf {
    let abs = if file.is_absolute() {
        file.to_path_buf()
    } else {
        let from_root = root.join(file);
        if from_root.exists() {
            from_root
        } else {
            cwd.join(file)
        }
    };
    normalize_path(&abs)
}

pub(crate) fn make_relative(abs: &Path, root: &Path) -> PathBuf {
    abs.strip_prefix(root).unwrap_or(abs).to_path_buf()
}

/// Render a path relative to `root` as a forward-slashed string for output, so
/// query JSON/paths match the rest of the CLI on every platform (including
/// Windows, where `Path::display` would otherwise use `\`).
pub(crate) fn rel_str(abs: &Path, root: &Path) -> String {
    make_relative(abs, root)
        .display()
        .to_string()
        .replace('\\', "/")
}

/// Parse a file's top-level exports and named imports. Error messages include
/// the path for context.
pub(crate) fn read_symbols(
    abs_file: &Path,
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<FileSymbols> {
    let source = sources
        .read_path(abs_file)
        .context(format!("reading {}", abs_file.display()))?;
    extract_symbols_at_path(abs_file, &source, is_tsx_file(abs_file))
        .context(format!("extracting symbols from {}", abs_file.display()))
}

#[cfg(test)]
mod tests;

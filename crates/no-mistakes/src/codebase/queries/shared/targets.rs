use super::{resolve_input_file, Target};
use crate::codebase::ts_resolver::normalize_path;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

/// Prepare a group of local-query targets under one request-owned session.
///
/// Inputs are normalized, sorted, and deduplicated before validation. This is
/// deliberately the only batch entrypoint so all target files share one
/// visible-file inventory and `SourceStore` for the duration of the request.
pub(crate) fn resolve_targets(
    files: &[PathBuf],
    root: Option<&Path>,
    tsconfig: Option<&Path>,
) -> Result<Vec<Target>> {
    anyhow::ensure!(!files.is_empty(), "at least one file is required");
    let cwd = std::env::current_dir().context("reading current directory")?;
    let root = normalize_path(&crate::cli::resolve_root(
        root.unwrap_or_else(|| Path::new(".")),
        &cwd,
    ));
    let session =
        crate::codebase::analysis_session::AnalysisSession::new(crate::diagnostics::current());
    let dataset = session.dataset(&root);
    let snapshot = dataset.visible_paths_arc();
    let sources = dataset.sources_for(&root);
    let explicit_tsconfig = tsconfig.map(|path| {
        normalize_path(&if path.is_absolute() {
            path.to_path_buf()
        } else {
            root.join(path)
        })
    });
    let visible_files = Arc::new(OnceLock::new());
    let mut inputs: Vec<(PathBuf, PathBuf)> = files
        .iter()
        .map(|file| (resolve_input_file(file, &root, &cwd), file.clone()))
        .collect();
    inputs.sort_by(|(left, _), (right, _)| left.cmp(right));
    inputs.dedup_by(|(left, _), (right, _)| left == right);

    // Validate the complete request before any source reads or parsing. This
    // prevents a partial report when a later input is a typo or a directory.
    for (abs_file, input) in &inputs {
        anyhow::ensure!(abs_file.is_file(), "not a file: {}", input.display());
    }

    Ok(inputs
        .into_iter()
        .map(|(abs_file, _)| Target {
            root: root.clone(),
            visible_paths: Arc::clone(&snapshot),
            abs_file,
            session: Arc::clone(&session),
            sources: Arc::clone(&sources),
            dataset: Arc::clone(&dataset),
            explicit_tsconfig: explicit_tsconfig.clone(),
            visible_files: Arc::clone(&visible_files),
            tsconfig: OnceLock::new(),
        })
        .collect())
}

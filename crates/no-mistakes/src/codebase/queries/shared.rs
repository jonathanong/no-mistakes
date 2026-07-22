use crate::codebase::dependencies::extract::is_tsx_file;
use crate::codebase::ts_resolver::{normalize_path, TsConfig};
use crate::codebase::ts_symbols::{extract_symbols_at_path, FileSymbols};
use anyhow::{Context, Result};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Resolved root, tsconfig, and the single absolute target file. Shared setup
/// for every lightweight query command so `--root`/`--tsconfig` fallback and
/// path normalization behave identically (and match `SymbolIndex` keys, which
/// are built from `normalize_path(root.join(rel))`).
pub(crate) struct Target {
    pub root: PathBuf,
    pub tsconfig: TsConfig,
    pub tsconfig_catalog: crate::codebase::ts_resolver::TsConfigCatalog,
    pub abs_file: PathBuf,
    pub visible_files: HashSet<PathBuf>,
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
    let visible_paths = crate::codebase::ts_source::discover_visible_paths(&root);
    let fallback_tsconfig = crate::codebase::ts_resolver::resolve_tsconfig_from_visible(
        tsconfig,
        &root,
        &visible_paths,
    )
    .or_else(|error| {
        if tsconfig.is_some() {
            Err(error)
        } else {
            Ok(TsConfig {
                dir: root.clone(),
                paths: Vec::new(),
                paths_dir: root.clone(),
                base_url: None,
            })
        }
    })?;
    let abs_file = resolve_input_file(file, &root, &cwd);
    // Reject a missing target or a directory up front so a typo or stale path
    // is an explicit error rather than an empty (and misleading) result.
    anyhow::ensure!(abs_file.is_file(), "not a file: {}", file.display());
    let tsconfig_catalog = match tsconfig {
        None => crate::codebase::ts_resolver::TsConfigCatalog::from_visible(
            &root,
            std::slice::from_ref(&root),
            &visible_paths,
        ),
        Some(path) => {
            let path = if path.is_absolute() {
                path.to_path_buf()
            } else {
                root.join(path)
            };
            crate::codebase::ts_resolver::TsConfigCatalog::forced(
                &root,
                fallback_tsconfig.clone(),
                Some(normalize_path(&path)),
            )
        }
    };
    let tsconfig = tsconfig_catalog.config_for(&abs_file).clone();
    let visible_files = visible_paths
        .into_iter()
        .map(|path| normalize_path(&path))
        .collect();
    Ok(Target {
        root,
        tsconfig,
        tsconfig_catalog,
        abs_file,
        visible_files,
    })
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
pub(crate) fn read_symbols(abs_file: &Path) -> Result<FileSymbols> {
    let source =
        std::fs::read_to_string(abs_file).context(format!("reading {}", abs_file.display()))?;
    extract_symbols_at_path(abs_file, &source, is_tsx_file(abs_file))
        .context(format!("extracting symbols from {}", abs_file.display()))
}

#[cfg(test)]
mod tests;

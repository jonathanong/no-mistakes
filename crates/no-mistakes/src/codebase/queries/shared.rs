use crate::codebase::dependencies::extract::is_tsx_file;
use crate::codebase::ts_resolver::{normalize_path, resolve_tsconfig, TsConfig};
use crate::codebase::ts_symbols::{extract_symbols, FileSymbols};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Resolved root, tsconfig, and the single absolute target file. Shared setup
/// for every lightweight query command so `--root`/`--tsconfig` fallback and
/// path normalization behave identically (and match `SymbolIndex` keys, which
/// are built from `normalize_path(root.join(rel))`).
pub(crate) struct Target {
    pub root: PathBuf,
    pub tsconfig: TsConfig,
    pub abs_file: PathBuf,
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
    let tsconfig = resolve_tsconfig(tsconfig, &root)?;
    let abs_file = resolve_input_file(file, &root, &cwd);
    // Reject a missing target up front so a typo or stale path is an explicit
    // error rather than an empty (and misleading) result.
    anyhow::ensure!(abs_file.exists(), "file not found: {}", file.display());
    Ok(Target {
        root,
        tsconfig,
        abs_file,
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

/// Render a path relative to `root` as a forward-slashed string for output.
pub(crate) fn rel_str(abs: &Path, root: &Path) -> String {
    make_relative(abs, root).display().to_string()
}

/// Parse a file's top-level exports and named imports. Error messages include
/// the path for context.
pub(crate) fn read_symbols(abs_file: &Path) -> Result<FileSymbols> {
    let source =
        std::fs::read_to_string(abs_file).context(format!("reading {}", abs_file.display()))?;
    extract_symbols(&source, is_tsx_file(abs_file))
        .context(format!("extracting symbols from {}", abs_file.display()))
}

#[cfg(test)]
mod tests;

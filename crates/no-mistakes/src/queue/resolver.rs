use anyhow::Result;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

const EXTENSIONS: &[&str] = &["mts", "ts", "tsx", "mjs", "js", "jsx", "cjs", "cts"];

#[derive(Debug, Clone, Default)]
pub(crate) struct TsConfig {
    pub paths_dir: PathBuf,
    pub base_url: Option<PathBuf>,
    pub paths: Vec<(String, Vec<String>)>,
}

impl From<&crate::codebase::ts_resolver::TsConfig> for TsConfig {
    fn from(config: &crate::codebase::ts_resolver::TsConfig) -> Self {
        Self {
            paths_dir: config.paths_dir.clone(),
            base_url: config.base_url.clone(),
            paths: config.paths.clone(),
        }
    }
}

pub(crate) fn load_tsconfig_from_visible(
    root: &Path,
    explicit: Option<&Path>,
    visible_paths: &[PathBuf],
) -> Result<TsConfig> {
    let config =
        crate::codebase::ts_resolver::resolve_tsconfig_from_visible(explicit, root, visible_paths)?;
    Ok(TsConfig::from(&config))
}

pub(crate) fn resolve_import_from_visible(
    specifier: &str,
    current_file: &Path,
    root: &Path,
    tsconfig: &TsConfig,
    visible_files: &HashSet<PathBuf>,
) -> Option<PathBuf> {
    resolve_import_inner(specifier, current_file, root, tsconfig, Some(visible_files))
}

pub(super) fn resolve_import_inner(
    specifier: &str,
    current_file: &Path,
    root: &Path,
    tsconfig: &TsConfig,
    visible_files: Option<&HashSet<PathBuf>>,
) -> Option<PathBuf> {
    if specifier.starts_with('.') {
        let parent = current_file.parent()?;
        return resolve_candidate(&parent.join(specifier), visible_files);
    }
    for (pattern, targets) in &tsconfig.paths {
        if let Some(capture) = match_pattern(pattern, specifier) {
            for target in targets {
                let replaced = target.replace('*', capture);
                let base = tsconfig
                    .base_url
                    .as_ref()
                    .unwrap_or(&tsconfig.paths_dir)
                    .join(replaced);
                if let Some(path) = resolve_candidate(&base, visible_files) {
                    return Some(path);
                }
            }
        }
    }
    if let Some(base_url) = &tsconfig.base_url {
        if let Some(path) = resolve_candidate(&base_url.join(specifier), visible_files) {
            return Some(path);
        }
    }
    resolve_candidate(&root.join(specifier), visible_files)
}

fn match_pattern<'a>(pattern: &str, specifier: &'a str) -> Option<&'a str> {
    if let Some((prefix, suffix)) = pattern.split_once('*') {
        let rest = specifier.strip_prefix(prefix)?;
        return rest.strip_suffix(suffix);
    }
    (pattern == specifier).then_some("")
}

fn resolve_candidate(path: &Path, visible_files: Option<&HashSet<PathBuf>>) -> Option<PathBuf> {
    if is_visible_file(path, visible_files) && is_source(path) {
        return Some(path.canonicalize().unwrap_or(path.to_path_buf()));
    }
    for ext in EXTENSIONS {
        let with_ext = path.with_extension(ext);
        if is_visible_file(&with_ext, visible_files) {
            return Some(with_ext.canonicalize().unwrap_or(with_ext));
        }
        let index = path.join(format!("index.{ext}"));
        if is_visible_file(&index, visible_files) {
            return Some(crate::codebase::ts_resolver::normalize_path(&index));
        }
    }
    None
}

fn is_visible_file(path: &Path, visible_files: Option<&HashSet<PathBuf>>) -> bool {
    visible_files.map_or_else(
        || path.is_file(),
        |visible| visible.contains(&crate::codebase::ts_resolver::normalize_path(path)),
    )
}

fn is_source(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some(ext) if EXTENSIONS.contains(&ext)
    )
}

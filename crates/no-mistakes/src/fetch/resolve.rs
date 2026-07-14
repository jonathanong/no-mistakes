use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub fn resolve_import(current_file: &Path, specifier: &str) -> Option<PathBuf> {
    resolve_import_inner(current_file, specifier, None)
}

pub(crate) fn resolve_import_from_visible(
    current_file: &Path,
    specifier: &str,
    visible_files: &HashSet<PathBuf>,
) -> Option<PathBuf> {
    resolve_import_inner(current_file, specifier, Some(visible_files))
}

fn resolve_import_inner(
    current_file: &Path,
    specifier: &str,
    visible_files: Option<&HashSet<PathBuf>>,
) -> Option<PathBuf> {
    const RUNTIME_EXTENSIONS: [&str; 4] = ["tsx", "ts", "jsx", "js"];

    if specifier.starts_with('.') {
        let parent = current_file.parent()?;
        let joined = parent.join(specifier);
        if is_visible_file(&joined, visible_files) {
            if !joined
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| RUNTIME_EXTENSIONS.contains(&ext))
            {
                return None;
            }
            return Some(crate::codebase::ts_resolver::normalize_path(&joined));
        }
        for ext in RUNTIME_EXTENSIONS {
            let path = joined.with_extension(ext);
            if is_visible_file(&path, visible_files) {
                return Some(crate::codebase::ts_resolver::normalize_path(&path));
            }
            let index = joined.join(format!("index.{ext}"));
            if is_visible_file(&index, visible_files) {
                return Some(crate::codebase::ts_resolver::normalize_path(&index));
            }
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

pub fn relative_string(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

pub fn is_client_route_file(path: &Path) -> anyhow::Result<bool> {
    use crate::ast;

    if !path.exists() {
        return Ok(false);
    }

    let source = std::fs::read_to_string(path)?;
    ast::with_program(path, &source, |program, _| {
        program
            .directives
            .iter()
            .any(|directive| directive.directive == "use client")
    })
}

#[cfg(test)]
mod tests;

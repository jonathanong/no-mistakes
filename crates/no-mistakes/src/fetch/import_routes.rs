use crate::fetch::file_facts::ParsedFileCache;
use crate::fetch::imports::{collect_imports, collect_imports_from_visible};
use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

pub fn is_route_handler_file(path: &Path) -> bool {
    path.file_stem().and_then(|stem| stem.to_str()) == Some("route")
}

pub fn route_reaches_target(
    path: &Path,
    target: &Path,
    visited: &mut HashSet<PathBuf>,
    import_cache: &mut HashMap<PathBuf, Vec<PathBuf>>,
) -> Result<bool> {
    route_reaches_target_with_visibility(path, target, visited, import_cache, None)
}

#[doc(hidden)]
pub fn route_reaches_target_from_visible(
    path: &Path,
    target: &Path,
    visited: &mut HashSet<PathBuf>,
    import_cache: &mut HashMap<PathBuf, Vec<PathBuf>>,
    visible_files: &HashSet<PathBuf>,
) -> Result<bool> {
    route_reaches_target_with_visibility(path, target, visited, import_cache, Some(visible_files))
}

#[doc(hidden)]
pub fn route_reaches_target_from_visible_with_facts(
    path: &Path,
    target: &Path,
    root: &Path,
    visited: &mut HashSet<PathBuf>,
    import_cache: &mut HashMap<PathBuf, Vec<PathBuf>>,
    parsed_files: &mut ParsedFileCache,
    visible_files: &HashSet<PathBuf>,
) -> Result<bool> {
    let abs_target = crate::codebase::ts_resolver::normalize_path(target);
    route_reaches_target_with_facts_inner(
        path,
        &abs_target,
        root,
        visited,
        import_cache,
        parsed_files,
        visible_files,
    )
}

fn route_reaches_target_with_facts_inner(
    path: &Path,
    abs_target: &Path,
    root: &Path,
    visited: &mut HashSet<PathBuf>,
    import_cache: &mut HashMap<PathBuf, Vec<PathBuf>>,
    parsed_files: &mut ParsedFileCache,
    visible_files: &HashSet<PathBuf>,
) -> Result<bool> {
    let abs_path = crate::codebase::ts_resolver::normalize_path(path);
    if abs_path == abs_target {
        return Ok(true);
    }
    if !visible_files.contains(&abs_path) || !visited.insert(abs_path.clone()) {
        return Ok(false);
    }

    let facts = parsed_files.load(&abs_path, root, import_cache, visible_files)?;
    for import in facts.imports {
        if route_reaches_target_with_facts_inner(
            &import,
            abs_target,
            root,
            visited,
            import_cache,
            parsed_files,
            visible_files,
        )? {
            return Ok(true);
        }
    }
    Ok(false)
}

fn route_reaches_target_with_visibility(
    path: &Path,
    target: &Path,
    visited: &mut HashSet<PathBuf>,
    import_cache: &mut HashMap<PathBuf, Vec<PathBuf>>,
    visible_files: Option<&HashSet<PathBuf>>,
) -> Result<bool> {
    let abs_target = match visible_files {
        Some(_) => crate::codebase::ts_resolver::normalize_path(target),
        None => target
            .canonicalize()
            .unwrap_or_else(|_| target.to_path_buf()),
    };
    route_reaches_target_inner(path, &abs_target, visited, import_cache, visible_files)
}

fn route_reaches_target_inner(
    path: &Path,
    abs_target: &Path,
    visited: &mut HashSet<PathBuf>,
    import_cache: &mut HashMap<PathBuf, Vec<PathBuf>>,
    visible_files: Option<&HashSet<PathBuf>>,
) -> Result<bool> {
    let abs_path = match visible_files {
        Some(_) => crate::codebase::ts_resolver::normalize_path(path),
        None => match path.canonicalize() {
            Ok(p) => p,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(false),
            Err(e) => return Err(e.into()),
        },
    };
    if abs_path == abs_target {
        return Ok(true);
    }
    if visible_files.is_some_and(|visible| !visible.contains(&abs_path)) {
        return Ok(false);
    }
    if visited.contains(&abs_path) {
        return Ok(false);
    }
    visited.insert(abs_path.clone());

    let imports = match visible_files {
        Some(visible) => collect_imports_from_visible(&abs_path, import_cache, visible)?,
        None => collect_imports(&abs_path, import_cache)?,
    };
    for import in imports {
        if route_reaches_target_inner(&import, abs_target, visited, import_cache, visible_files)? {
            return Ok(true);
        }
    }

    Ok(false)
}

pub fn collect_layout_chain_files(route_file: &Path, frontend_root: &Path) -> Vec<PathBuf> {
    collect_layout_chain_files_inner(route_file, frontend_root, None)
}

#[doc(hidden)]
pub fn collect_layout_chain_files_from_visible(
    route_file: &Path,
    frontend_root: &Path,
    visible_files: &HashSet<PathBuf>,
) -> Vec<PathBuf> {
    collect_layout_chain_files_inner(route_file, frontend_root, Some(visible_files))
}

fn collect_layout_chain_files_inner(
    route_file: &Path,
    frontend_root: &Path,
    visible_files: Option<&HashSet<PathBuf>>,
) -> Vec<PathBuf> {
    let mut layout_files = Vec::new();
    let mut current = route_file.parent();
    while let Some(parent) = current {
        if !parent.starts_with(frontend_root) {
            break;
        }

        for stem in ["layout", "loading", "error", "not-found", "template"] {
            for ext in ["tsx", "ts", "jsx", "js"] {
                let layout_file = parent.join(format!("{stem}.{ext}"));
                if visible_files.map_or_else(
                    || layout_file.is_file(),
                    |visible| visible.contains(&layout_file),
                ) {
                    layout_files.push(layout_file);
                }
            }
        }

        current = parent.parent();
    }

    layout_files
}

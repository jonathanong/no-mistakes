use super::ImportUsagesArgs;
use crate::codebase::ts_source::{discover_files, relative_slash_path};
use anyhow::Result;
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::path::{Path, PathBuf};

pub fn resolve_files(args: &ImportUsagesArgs, root: &Path, cwd: &Path) -> Result<Vec<PathBuf>> {
    let mut files = if args.files.is_empty() {
        scan_files(root, &args.scan_roots)
    } else {
        args.files
            .iter()
            .map(|file| normalize_input_path(file, root, cwd))
            .collect()
    };
    files.sort();
    files.dedup();
    let filter = file_filter(&args.filters)?;
    if let Some(filter) = filter {
        files.retain(|path| filter.is_match(relative_slash_path(root, path)));
    }
    Ok(files)
}

pub fn roots_for_output(args: &ImportUsagesArgs, root: &Path) -> Vec<String> {
    if args.files.is_empty() && args.scan_roots.is_empty() {
        return vec![root.display().to_string()];
    }
    if args.files.is_empty() {
        return args
            .scan_roots
            .iter()
            .map(|path| path.display().to_string())
            .collect();
    }
    args.files
        .iter()
        .map(|path| path.display().to_string())
        .collect()
}

pub fn normalize_root(root: Option<&Path>, cwd: &Path) -> PathBuf {
    let root = root.unwrap_or(cwd);
    crate::codebase::ts_resolver::normalize_path(root)
}

fn scan_files(root: &Path, scan_roots: &[PathBuf]) -> Vec<PathBuf> {
    if scan_roots.is_empty() {
        return discover_files(root, &[]);
    }
    scan_roots
        .iter()
        .flat_map(|scan_root| {
            let abs = if scan_root.is_absolute() {
                scan_root.clone()
            } else {
                root.join(scan_root)
            };
            discover_files(&abs, &[])
        })
        .collect()
}

fn file_filter(patterns: &[String]) -> Result<Option<GlobSet>> {
    if patterns.is_empty() {
        return Ok(None);
    }
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let pattern = pattern
            .strip_suffix('/')
            .map(|p| format!("{p}/**"))
            .unwrap_or_else(|| pattern.clone());
        builder.add(Glob::new(&pattern)?);
    }
    Ok(Some(builder.build()?))
}

fn normalize_input_path(path: &Path, root: &Path, cwd: &Path) -> PathBuf {
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else if root.is_absolute() {
        root.join(path)
    } else {
        cwd.join(root).join(path)
    };
    crate::codebase::ts_resolver::normalize_path(&path)
}

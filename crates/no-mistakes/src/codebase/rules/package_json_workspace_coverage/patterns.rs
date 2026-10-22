use super::RuleFinding;
use super::RULE_ID;
use crate::codebase::glob_normalize;
use crate::codebase::ts_source::relative_slash_path;
use anyhow::Result;
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub(super) fn config_finding(file: &str, detail: &str) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line: 1,
        message: format!("{file}: {detail}"),
        import: None,
        target: None,
    }
}

pub(super) fn workspace_config_file(root: &Path, files: &[PathBuf]) -> String {
    let pnpm = crate::codebase::ts_resolver::normalize_path(&root.join("pnpm-workspace.yaml"));
    if files
        .iter()
        .any(|path| crate::codebase::ts_resolver::normalize_path(path) == pnpm)
    {
        "pnpm-workspace.yaml".to_string()
    } else {
        "package.json".to_string()
    }
}

pub(super) fn package_directories(root: &Path, files: &[PathBuf]) -> Vec<String> {
    files
        .iter()
        .filter(|path| path.file_name().and_then(|name| name.to_str()) == Some("package.json"))
        .filter_map(|path| path.parent())
        .map(|dir| relative_slash_path(root, dir))
        .collect()
}

pub(super) fn unmatched_patterns(
    patterns: &[String],
    package_dirs: &[String],
) -> Result<Vec<String>> {
    let mut missing = Vec::new();
    for pattern in patterns {
        if pattern.starts_with('!') {
            continue;
        }
        let normalized = normalize_relative_path(pattern);
        let glob = GlobBuilder::new(&normalized)
            .literal_separator(true)
            .build()?;
        let matcher = glob.compile_matcher();
        if !package_dirs.iter().any(|dir| matcher.is_match(dir)) && !missing.contains(pattern) {
            missing.push(pattern.clone());
        }
    }
    missing.sort();
    Ok(missing)
}

pub(super) fn covered_workspace_dirs(
    root: &Path,
    files: &[PathBuf],
    workspace_globs: &[String],
) -> Result<HashSet<String>> {
    let include = build_workspace_globset(workspace_globs, false)?;
    let exclude = build_workspace_globset(workspace_globs, true)?;
    Ok(files
        .iter()
        .filter(|path| path.file_name().and_then(|name| name.to_str()) == Some("package.json"))
        .filter_map(|path| path.parent())
        .filter_map(|dir| {
            let rel = relative_slash_path(root, dir);
            (include.is_match(rel.as_str()) && !exclude.is_match(rel.as_str())).then_some(rel)
        })
        .collect())
}

pub(super) fn build_workspace_globset(patterns: &[String], excluded: bool) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let pattern = if excluded {
            let Some(stripped) = pattern.strip_prefix('!') else {
                continue;
            };
            stripped
        } else if pattern.starts_with('!') {
            continue;
        } else {
            pattern.as_str()
        };
        builder.add(
            GlobBuilder::new(&normalize_relative_path(pattern))
                .literal_separator(true)
                .build()?,
        );
    }
    Ok(builder.build()?)
}

pub(super) fn path_under_package_roots(path: &str, package_roots: &[String]) -> bool {
    package_roots.iter().any(|root| {
        let root = normalize_relative_path(root);
        !root.is_empty() && (path == root || path.starts_with(&format!("{root}/")))
    })
}

pub(super) fn normalize_relative_path(path: &str) -> String {
    glob_normalize::normalize(path)
}

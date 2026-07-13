use super::options::{parse_options, resolve_project_root, to_napi_error};
use no_mistakes::codebase::lockfile::{self, PackageManager};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

mod git;
use git::{detect_lockfiles_from_head, find_git_root, git_ref_exists, git_show_file};

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct LockfileDiffOptions {
    pub root: Option<String>,
    pub base: Option<String>,
    pub head: Option<String>,
    pub lockfile: Option<String>,
}

#[derive(Serialize)]
struct LockfileDiffEntry {
    lockfile: String,
    manager: String,
    added: Vec<String>,
    removed: Vec<String>,
    changed: Vec<String>,
}

pub(crate) fn lockfile_diff_json_impl(options_json: String) -> napi::Result<String> {
    let options = parse_options::<LockfileDiffOptions>(&options_json)?;
    let base = options.base.filter(|s| !s.is_empty()).ok_or_else(|| {
        napi::Error::from_reason(
            "`base` is required; pass a git ref such as `\"HEAD\"` or `\"main\"`",
        )
    })?;
    let root = resolve_project_root(options.root.as_deref()).map_err(to_napi_error)?;
    let root = root.canonicalize().unwrap_or(root);

    let git_root = find_git_root(&root).unwrap_or_else(|| root.clone());

    let lf_paths: Vec<PathBuf> = if let Some(lf) = options.lockfile {
        vec![root.join(lf)]
    } else if let Some(head) = options.head.as_deref() {
        if !git_ref_exists(&git_root, head) {
            return Err(napi::Error::from_reason(format!(
                "head ref `{}` does not exist; ensure it exists in the git history",
                head
            )));
        }
        // Detect from head commit so newly added lockfiles are found even when
        // the working tree is still at a different commit.
        detect_lockfiles_from_head(&git_root, head, &root)
    } else {
        let visible_paths = no_mistakes::codebase::ts_source::discover_visible_paths(&root);
        [
            "pnpm-lock.yaml",
            "package-lock.json",
            "npm-shrinkwrap.json",
            "yarn.lock",
            "bun.lock",
        ]
        .iter()
        .map(|name| no_mistakes::codebase::ts_resolver::normalize_path(&root.join(name)))
        .filter(|candidate| {
            visible_paths
                .iter()
                .any(|path| no_mistakes::codebase::ts_resolver::normalize_path(path) == *candidate)
        })
        .collect()
    };

    let mut entries = Vec::new();

    for lf_path in &lf_paths {
        let basename = lf_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let Some(manager) = lockfile::detect_manager(basename) else {
            if lockfile::is_binary_lockfile(basename) {
                return Err(napi::Error::from_reason(format!(
                    "`{}` is a binary lockfile and cannot be parsed for dependency changes",
                    basename
                )));
            }
            continue;
        };
        let rel = lf_path
            .strip_prefix(&git_root)
            .unwrap_or(lf_path)
            .to_string_lossy()
            .replace('\\', "/");
        let new_content = if let Some(head) = options.head.as_deref() {
            match git_show_file(&git_root, head, &rel) {
                Some(content) => content,
                None => {
                    if !git_ref_exists(&git_root, head) {
                        return Err(napi::Error::from_reason(format!(
                            "Could not retrieve `{}` at ref `{}`; ensure the head ref exists in the git history",
                            rel, head
                        )));
                    }
                    // Ref is valid but file deleted at head — report all packages as removed
                    String::new()
                }
            }
        } else {
            std::fs::read_to_string(lf_path).unwrap_or_default()
        };
        let old_content = if options.head.is_some() {
            match git_show_file(&git_root, &base, &rel) {
                Some(content) => content,
                None => {
                    if !git_ref_exists(&git_root, &base) {
                        return Err(napi::Error::from_reason(format!(
                            "Could not retrieve `{}` at ref `{}`; ensure the base ref exists in the git history",
                            rel, base
                        )));
                    }
                    // Valid base ref but file not at base — newly added at head
                    String::new()
                }
            }
        } else {
            match git_show_file(&git_root, &base, &rel) {
                Some(c) => c,
                None => {
                    if !git_ref_exists(&git_root, &base) {
                        return Err(napi::Error::from_reason(format!(
                            "Could not retrieve `{}` at ref `{}`; ensure the base ref exists in the git history",
                            rel, base
                        )));
                    }
                    // Valid base ref but file not at base — newly added lockfile; report all as added
                    String::new()
                }
            }
        };
        let old_pkgs = lockfile::parse_lockfile(manager, &old_content);
        let new_pkgs = lockfile::parse_lockfile(manager, &new_content);
        let diff = lockfile::diff(&old_pkgs, &new_pkgs);
        entries.push(LockfileDiffEntry {
            lockfile: rel,
            manager: manager_name(manager).to_string(),
            added: diff.added,
            removed: diff.removed,
            changed: diff.changed,
        });
    }

    serde_json::to_string_pretty(&entries).map_err(|e| napi::Error::from_reason(e.to_string()))
}

fn manager_name(m: PackageManager) -> &'static str {
    match m {
        PackageManager::Npm => "npm",
        PackageManager::Pnpm => "pnpm",
        PackageManager::Yarn => "yarn",
        PackageManager::Bun => "bun",
    }
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests2;

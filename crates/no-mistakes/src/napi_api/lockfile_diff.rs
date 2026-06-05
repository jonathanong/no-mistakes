use super::options::{parse_options, resolve_project_root, to_napi_error};
use no_mistakes::codebase::lockfile::{self, PackageManager};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

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
        // Detect from head commit so newly added lockfiles are found even when
        // the working tree is still at a different commit.
        detect_lockfiles_from_head(&git_root, head, &root)
    } else {
        [
            "pnpm-lock.yaml",
            "package-lock.json",
            "npm-shrinkwrap.json",
            "yarn.lock",
            "bun.lock",
        ]
        .iter()
        .map(|n| root.join(n))
        .filter(|p| p.exists())
        .collect()
    };

    let mut entries = Vec::new();

    for lf_path in &lf_paths {
        let basename = lf_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let Some(manager) = lockfile::detect_manager(basename) else {
            continue;
        };
        let rel = lf_path
            .strip_prefix(&git_root)
            .unwrap_or(lf_path)
            .to_string_lossy()
            .replace('\\', "/");
        let new_content = if let Some(head) = options.head.as_deref() {
            git_show_file(&git_root, head, &rel).ok_or_else(|| {
                napi::Error::from_reason(format!(
                    "Could not retrieve `{}` at ref `{}`; ensure the head ref exists in the git history",
                    rel, head
                ))
            })?
        } else {
            std::fs::read_to_string(lf_path).unwrap_or_default()
        };
        // When head is supplied, the file was detected from the head commit and may not exist
        // at base (newly added lockfile) — treat missing base as empty baseline.
        // Without head (disk-based), a missing base file means an invalid ref; return an error.
        let old_content = if options.head.is_some() {
            git_show_file(&git_root, &base, &rel).unwrap_or_default()
        } else {
            git_show_file(&git_root, &base, &rel).ok_or_else(|| {
                napi::Error::from_reason(format!(
                    "Could not retrieve `{}` at ref `{}`; ensure the base ref exists in the git history",
                    rel, base
                ))
            })?
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

fn detect_lockfiles_from_head(git_root: &Path, head: &str, root: &Path) -> Vec<PathBuf> {
    let candidates = [
        "pnpm-lock.yaml",
        "package-lock.json",
        "npm-shrinkwrap.json",
        "yarn.lock",
        "bun.lock",
    ];
    let root_rel = root
        .strip_prefix(git_root)
        .unwrap_or(std::path::Path::new(""))
        .to_string_lossy()
        .replace('\\', "/");
    candidates
        .iter()
        .filter(|name| {
            let rel = if root_rel.is_empty() {
                name.to_string()
            } else {
                format!("{}/{}", root_rel, name)
            };
            git_show_file(git_root, head, &rel).is_some()
        })
        .map(|name| root.join(name))
        .collect()
}

fn find_git_root(dir: &Path) -> Option<PathBuf> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(dir)
        .output()
        .ok()?;
    if output.status.success() {
        let s = String::from_utf8(output.stdout).ok()?;
        Some(PathBuf::from(s.trim()))
    } else {
        None
    }
}

fn git_show_file(root: &Path, git_ref: &str, rel_path: &str) -> Option<String> {
    let output = std::process::Command::new("git")
        .args(["show", &format!("{}:{}", git_ref, rel_path)])
        .current_dir(root)
        .output()
        .ok()?;
    if output.status.success() {
        String::from_utf8(output.stdout).ok()
    } else {
        None
    }
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

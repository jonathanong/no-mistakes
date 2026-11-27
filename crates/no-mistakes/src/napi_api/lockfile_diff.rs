use super::options::{parse_options_value, resolve_project_root, to_napi_error};
use no_mistakes::codebase::lockfile::{self, PackageManager};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

mod git;
use git::{detect_lockfiles_from_head, find_git_root, git_ref_exists, git_show_file};

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct LockfileDiffOptions {
    pub root: Option<String>,
    pub base: Option<String>,
    pub head: Option<String>,
    pub lockfile: Option<String>,
    pub include_warnings: bool,
}

#[derive(Serialize)]
struct LockfileDiffEntry {
    lockfile: String,
    manager: String,
    added: Vec<String>,
    removed: Vec<String>,
    changed: Vec<String>,
}

#[derive(Serialize)]
struct LockfileDiffWarning {
    code: &'static str,
    lockfile: String,
    message: String,
}

fn git_io_to_napi(error: std::io::Error) -> napi::Error {
    to_napi_error(error.into())
}

pub(crate) fn lockfile_diff_json_impl(options: serde_json::Value) -> napi::Result<String> {
    let options = parse_options_value::<LockfileDiffOptions>(options)?;
    let base = options.base.filter(|s| !s.is_empty()).ok_or_else(|| {
        napi::Error::from_reason(
            "`base` is required; pass a git ref such as `\"HEAD\"` or `\"main\"`",
        )
    })?;
    let root = resolve_project_root(options.root.as_deref()).map_err(to_napi_error)?;
    let root = root.canonicalize().unwrap_or(root);

    let git_root = find_git_root(&root)
        .map_err(git_io_to_napi)?
        .unwrap_or_else(|| root.clone());

    let lf_paths: Vec<PathBuf> = if let Some(lf) = options.lockfile {
        vec![root.join(lf)]
    } else if let Some(head) = options.head.as_deref() {
        if !git_ref_exists(&git_root, head).map_err(git_io_to_napi)? {
            return Err(napi::Error::from_reason(format!(
                "head ref `{}` does not exist; ensure it exists in the git history",
                head
            )));
        }
        // Detect from head commit so newly added lockfiles are found even when
        // the working tree is still at a different commit.
        detect_lockfiles_from_head(&git_root, head, &root).map_err(git_io_to_napi)?
    } else {
        let visible_paths = no_mistakes::codebase::ts_source::discover_visible_paths(&root);
        lockfile::LOCKFILE_BASENAMES
            .iter()
            .map(|name| no_mistakes::codebase::ts_resolver::normalize_path(&root.join(name)))
            .filter(|candidate| {
                visible_paths.iter().any(|path| {
                    no_mistakes::codebase::ts_resolver::normalize_path(path) == *candidate
                })
            })
            .collect()
    };

    let mut entries = Vec::new();
    let mut warnings = Vec::new();
    for lf_path in &lf_paths {
        match diff_one_lockfile(lf_path, &git_root, options.head.as_deref(), &base)? {
            LockfileOutcome::Entry(entry) => entries.push(entry),
            LockfileOutcome::Warning(warning) if options.include_warnings => {
                warnings.push(warning);
            }
            LockfileOutcome::Warning(warning) => {
                return Err(napi::Error::from_reason(warning.message));
            }
            LockfileOutcome::Unsupported => {}
        }
    }

    if options.include_warnings {
        Ok(crate::cli::json_string(&serde_json::json!({
            "entries": entries,
            "warnings": warnings,
        })))
    } else {
        Ok(crate::cli::json_string(&entries))
    }
}

fn git_lockfile_content(
    git_root: &Path,
    reference: &str,
    lockfile: &str,
    kind: &str,
) -> napi::Result<String> {
    if let Some(content) = git_show_file(git_root, reference, lockfile).map_err(git_io_to_napi)? {
        return Ok(content);
    }
    if !git_ref_exists(git_root, reference).map_err(git_io_to_napi)? {
        return Err(napi::Error::from_reason(format!(
            "Could not retrieve `{lockfile}` at ref `{reference}`; ensure the {kind} ref exists in the git history"
        )));
    }
    Ok(String::new())
}

enum LockfileOutcome {
    Entry(LockfileDiffEntry),
    Warning(LockfileDiffWarning),
    Unsupported,
}

fn diff_one_lockfile(
    lf_path: &Path,
    git_root: &Path,
    head: Option<&str>,
    base: &str,
) -> napi::Result<LockfileOutcome> {
    let basename = lf_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let rel = lf_path
        .strip_prefix(git_root)
        .unwrap_or(lf_path)
        .to_string_lossy()
        .replace('\\', "/");
    let Some(manager) = lockfile::detect_manager(basename) else {
        return Ok(if lockfile::is_binary_lockfile(basename) {
            LockfileOutcome::Warning(LockfileDiffWarning {
                code: "binary-lockfile",
                lockfile: rel,
                message: lockfile::binary_lockfile_warning(basename),
            })
        } else {
            LockfileOutcome::Unsupported
        });
    };
    let new_content = match head {
        Some(head) => git_lockfile_content(git_root, head, &rel, "head")?,
        None => std::fs::read_to_string(lf_path).unwrap_or_default(),
    };
    let old_content = git_lockfile_content(git_root, base, &rel, "base")?;
    let old_pkgs = packages_for_entry(manager, &rel, &old_content)?;
    let new_pkgs = packages_for_entry(manager, &rel, &new_content)?;
    let diff = lockfile::diff(&old_pkgs, &new_pkgs);
    Ok(LockfileOutcome::Entry(LockfileDiffEntry {
        lockfile: rel,
        manager: manager_name(manager).to_string(),
        added: diff.added,
        removed: diff.removed,
        changed: diff.changed,
    }))
}

fn packages_for_entry(
    manager: PackageManager,
    rel: &str,
    content: &str,
) -> napi::Result<Vec<lockfile::ResolvedPackage>> {
    lockfile::packages_for_diff(manager, content)
        .map_err(|error| napi::Error::from_reason(format!("`{rel}` has {}", error.description)))
}

fn manager_name(m: PackageManager) -> &'static str {
    m.as_str()
}

#[cfg(test)]
mod tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod tests2;
#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod tests3;

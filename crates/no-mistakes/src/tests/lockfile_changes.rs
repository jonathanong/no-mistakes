use super::{PlanArgs, Warning};
use no_mistakes::codebase::lockfile::{self, LockfileDiff};
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub(crate) struct LockfileAnalysis {
    pub diff_by_lockfile: Vec<(PathBuf, LockfileDiff)>,
    pub warnings: Vec<Warning>,
    pub fallback_triggered: bool,
}

pub(crate) fn analyze_lockfile_changes(
    args: &PlanArgs,
    root: &Path,
    all_files: &[PathBuf],
) -> LockfileAnalysis {
    let mut diff_by_lockfile = Vec::new();
    let mut warnings = Vec::new();
    let mut fallback_triggered = false;

    let git_root = find_git_root(root).unwrap_or_else(|| root.to_path_buf());

    for file in all_files {
        let basename = file.file_name().and_then(|n| n.to_str()).unwrap_or("");

        if lockfile::is_binary_lockfile(basename) {
            let rel = crate::tests::plan::relative_path(root, file);
            warnings.push(Warning {
                r#type: "lockfile-binary-unsupported".to_string(),
                message: format!(
                    "`{}` is a binary lockfile and cannot be analyzed for package changes; full-suite selection requires global fallback opt-in",
                    rel
                ),
                file: rel,
            });
            fallback_triggered = true;
            continue;
        }

        let Some(manager) = lockfile::detect_manager(basename) else {
            continue;
        };

        let git_rel = file
            .strip_prefix(&git_root)
            .unwrap_or(file.as_path())
            .to_string_lossy()
            .replace('\\', "/");

        let new_content = if let Some(head) = args.head.as_deref() {
            match git_show_file(&git_root, head, &git_rel) {
                Some(content) => content,
                None => {
                    let rel = crate::tests::plan::relative_path(root, file);
                    if !git_ref_exists(&git_root, head) {
                        warnings.push(Warning {
                            r#type: "lockfile-no-baseline".to_string(),
                            message: format!(
                                "Could not read `{}` at head ref `{}`; full-suite selection requires global fallback opt-in",
                                rel, head
                            ),
                            file: rel,
                        });
                        fallback_triggered = true;
                        continue;
                    }
                    // Valid head ref but file deleted at head — treat new content as empty
                    String::new()
                }
            }
        } else if is_diff_only_mode(args) {
            // In diff-only mode (--diff/--diff-stdin/etc.) without --head, the working tree
            // may still be at the base. Reading from disk would compare base-vs-base and miss
            // the lockfile change; fall back instead of producing a bogus empty diff.
            let rel = crate::tests::plan::relative_path(root, file);
            warnings.push(Warning {
                r#type: "lockfile-no-baseline".to_string(),
                message: format!(
                    "Could not determine new content of `{}` in diff-only mode. Provide `--head` to enable targeted lockfile analysis; full-suite selection requires global fallback opt-in.",
                    rel
                ),
                file: rel,
            });
            fallback_triggered = true;
            continue;
        } else {
            std::fs::read_to_string(file).unwrap_or_default()
        };
        let new_packages = lockfile::parse_lockfile(manager, &new_content);

        match args.base.as_deref() {
            Some(base) => match git_show_file(&git_root, base, &git_rel) {
                Some(old) => {
                    let old_packages = lockfile::parse_lockfile(manager, &old);
                    let lf_diff = lockfile::diff(&old_packages, &new_packages);
                    if !lf_diff.is_empty() {
                        diff_by_lockfile.push((file.clone(), lf_diff));
                    }
                }
                None => {
                    if !git_ref_exists(&git_root, base) {
                        // Invalid base ref — cannot determine what changed
                        let rel = crate::tests::plan::relative_path(root, file);
                        warnings.push(Warning {
                            r#type: "lockfile-no-baseline".to_string(),
                            message: format!(
                                "Could not read `{}` at base ref `{}`; full-suite selection requires global fallback opt-in",
                                rel, base
                            ),
                            file: rel,
                        });
                        fallback_triggered = true;
                    } else {
                        // Valid base ref but file not at base — newly added lockfile;
                        // treat baseline as empty so all packages are seen as added.
                        let old_packages = lockfile::parse_lockfile(manager, "");
                        let lf_diff = lockfile::diff(&old_packages, &new_packages);
                        if !lf_diff.is_empty() {
                            diff_by_lockfile.push((file.clone(), lf_diff));
                        }
                    }
                }
            },
            None => {
                let rel = crate::tests::plan::relative_path(root, file);
                warnings.push(Warning {
                r#type: "lockfile-no-baseline".to_string(),
                message: format!(
                        "Could not determine old content of `{}`. Provide `--base` to enable targeted lockfile analysis; full-suite selection requires global fallback opt-in.",
                        rel
                    ),
                    file: rel,
                });
                fallback_triggered = true;
            }
        }
    }

    LockfileAnalysis {
        diff_by_lockfile,
        warnings,
        fallback_triggered,
    }
}

fn is_diff_only_mode(args: &PlanArgs) -> bool {
    args.head.is_none() && super::changed_files::has_explicit_diff_source(args)
}

pub(super) fn find_git_root(dir: &Path) -> Option<PathBuf> {
    let mut command = std::process::Command::new("git");
    command
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(dir);
    let output = crate::invocation::command_output(&mut command).ok()?;
    if output.status.success() {
        let s = String::from_utf8(output.stdout).ok()?;
        Some(PathBuf::from(s.trim()))
    } else {
        None
    }
}

fn git_ref_exists(root: &Path, git_ref: &str) -> bool {
    let mut command = std::process::Command::new("git");
    command
        .args(["rev-parse", "--verify", git_ref])
        .current_dir(root);
    crate::invocation::command_output(&mut command)
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub(crate) fn git_show_file(root: &Path, git_ref: &str, rel_path: &str) -> Option<String> {
    let mut command = std::process::Command::new("git");
    command
        .args(["show", &format!("{}:{}", git_ref, rel_path)])
        .current_dir(root);
    let output = crate::invocation::command_output(&mut command).ok()?;
    if output.status.success() {
        String::from_utf8(output.stdout).ok()
    } else {
        None
    }
}

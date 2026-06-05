use super::{PlanArgs, Warning};
use no_mistakes::codebase::lockfile::{self, LockfileDiff, PackageManager};
use std::path::{Path, PathBuf};

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

    for file in all_files {
        let basename = file.file_name().and_then(|n| n.to_str()).unwrap_or("");

        if lockfile::is_binary_lockfile(basename) {
            let rel = crate::tests::plan::relative_path(root, file);
            warnings.push(Warning {
                r#type: "lockfile-binary-unsupported".to_string(),
                message: format!(
                    "`{}` is a binary lockfile and cannot be analyzed for package changes; falling back to full test suite",
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

        let new_content = std::fs::read_to_string(file).unwrap_or_default();
        let new_packages = lockfile::parse_lockfile(manager, &new_content);

        match get_old_lockfile_content(args, root, file, manager) {
            Some(old) => {
                let old_packages = lockfile::parse_lockfile(manager, &old);
                let lf_diff = lockfile::diff(&old_packages, &new_packages);
                if !lf_diff.is_empty() {
                    diff_by_lockfile.push((file.clone(), lf_diff));
                }
            }
            None => {
                let rel = crate::tests::plan::relative_path(root, file);
                warnings.push(Warning {
                    r#type: "lockfile-no-baseline".to_string(),
                    message: format!(
                        "Could not determine old content of `{}`; falling back to full test suite. Provide `--base` to enable targeted lockfile analysis.",
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

fn get_old_lockfile_content(
    args: &PlanArgs,
    root: &Path,
    lockfile_path: &Path,
    _manager: PackageManager,
) -> Option<String> {
    let base = args.base.as_deref()?;
    let rel = crate::tests::plan::relative_path(root, lockfile_path);
    git_show_file(root, base, &rel)
}

pub(crate) fn git_show_file(root: &Path, git_ref: &str, rel_path: &str) -> Option<String> {
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

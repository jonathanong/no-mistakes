use super::PlanArgs;
use anyhow::{Context, Result};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn collect_changed_files(args: &PlanArgs, root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for f in &args.changed_file {
        let path = if f.is_absolute() {
            f.clone()
        } else {
            root.join(f)
        };
        let resolved = path
            .canonicalize()
            .unwrap_or_else(|_| no_mistakes::codebase::ts_resolver::normalize_path(&path));
        files.push(resolved);
    }

    if let Some(ref path) = args.changed_files {
        let content = fs::read_to_string(path).with_context(|| {
            format!("Failed to read changed-files list from {}", path.display())
        })?;
        for line in content.lines() {
            let line = line.trim();
            if !line.is_empty() {
                let p = PathBuf::from(line);
                let path = if p.is_absolute() { p } else { root.join(p) };
                let resolved = path
                    .canonicalize()
                    .unwrap_or_else(|_| no_mistakes::codebase::ts_resolver::normalize_path(&path));
                files.push(resolved);
            }
        }
    }

    if args.base.is_some() || (args.changed_file.is_empty() && args.changed_files.is_none()) {
        match get_git_changed_files(root, args.base.as_deref(), args.head.as_deref()) {
            Ok(git_files) => {
                for f in git_files {
                    files.push(root.join(f));
                }
            }
            Err(e) => {
                if args.base.is_some() {
                    return Err(e);
                }
                eprintln!("warning: failed to retrieve changed files from git: {}", e);
            }
        }
    }

    let mut unique = HashSet::new();
    let mut result = Vec::new();
    for f in files {
        let normalized = no_mistakes::codebase::ts_resolver::normalize_path(&f);
        if unique.insert(normalized.clone()) {
            result.push(normalized);
        }
    }

    Ok(result)
}

fn get_git_changed_files(
    root: &Path,
    base: Option<&str>,
    head: Option<&str>,
) -> Result<Vec<PathBuf>> {
    let mut changed = HashSet::new();

    if let Some(base_commit) = base {
        let head_commit = head.unwrap_or("HEAD");
        let output = run_git(
            &[
                "diff",
                "--relative",
                "--name-only",
                &format!("{}...{}", base_commit, head_commit),
            ],
            root,
        )?;
        for line in output.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                changed.insert(PathBuf::from(trimmed));
            }
        }
    } else {
        for args in [
            &["diff", "--relative", "--name-only"][..],
            &["diff", "--cached", "--relative", "--name-only"][..],
            &["ls-files", "--others", "--exclude-standard"][..],
        ] {
            if let Ok(output) = run_git(args, root) {
                for line in output.lines() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        changed.insert(PathBuf::from(trimmed));
                    }
                }
            }
        }
    }

    let mut result: Vec<_> = changed.into_iter().collect();
    result.sort();
    Ok(result)
}

fn run_git(args: &[&str], root: &Path) -> Result<String> {
    let output = std::process::Command::new("git")
        .args(args)
        .current_dir(root)
        .output()?;
    if !output.status.success() {
        anyhow::bail!(
            "git command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(String::from_utf8(output.stdout)?)
}

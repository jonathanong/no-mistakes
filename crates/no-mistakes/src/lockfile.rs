use anyhow::Result;
use clap::{Args, Subcommand};
use no_mistakes::codebase::lockfile::{self, PackageManager};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Args)]
pub(crate) struct LockfileArgs {
    #[command(subcommand)]
    pub command: LockfileCommand,
}

#[derive(Subcommand)]
pub(crate) enum LockfileCommand {
    /// Show which packages changed between two lockfile versions.
    Diff(LockfileDiffArgs),
}

#[derive(Args)]
pub(crate) struct LockfileDiffArgs {
    /// Base git ref (e.g. main, HEAD~1).
    #[arg(long)]
    pub base: String,
    /// Head git ref (defaults to HEAD).
    #[arg(long)]
    pub head: Option<String>,
    /// Path to lockfile (relative to root). Auto-detected if omitted.
    #[arg(long)]
    pub lockfile: Option<PathBuf>,
    /// Root directory.
    #[arg(long, default_value = ".")]
    pub root: PathBuf,
    /// Output format: json (default) or paths.
    #[arg(long, default_value = "json")]
    pub format: String,
}

#[derive(Serialize)]
struct LockfileDiffOutput {
    lockfile: String,
    manager: String,
    added: Vec<String>,
    removed: Vec<String>,
    changed: Vec<String>,
}

pub(crate) fn run(args: LockfileArgs) -> Result<ExitCode> {
    match args.command {
        LockfileCommand::Diff(sub) => run_diff(sub),
    }
}

fn run_diff(args: LockfileDiffArgs) -> Result<ExitCode> {
    let cwd = std::env::current_dir()?;
    let root = no_mistakes::cli::resolve_optional_root(Some(&args.root), &cwd);
    let root = no_mistakes::codebase::ts_resolver::normalize_path(&root);
    let root = root.canonicalize().unwrap_or(root);

    // Use the git repository root for `git show` so subdirectory --root values
    // produce repo-relative paths (e.g. `packages/api/pnpm-lock.yaml` not `pnpm-lock.yaml`).
    let git_root = find_git_root(&root).unwrap_or_else(|| root.clone());

    let lockfile_paths = if let Some(lf) = args.lockfile {
        vec![root.join(lf)]
    } else if let Some(head) = args.head.as_deref() {
        // When --head is supplied, detect from the head commit so we find lockfiles
        // added or removed between base and head that don't exist on disk.
        detect_lockfiles_from_head(&git_root, head, &root)
    } else {
        detect_lockfiles_in_root(&root)
    };

    let mut outputs = Vec::new();

    for lf_path in &lockfile_paths {
        let basename = lf_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let Some(manager) = lockfile::detect_manager(basename) else {
            continue;
        };
        let rel = lf_path
            .strip_prefix(&git_root)
            .unwrap_or(lf_path)
            .to_string_lossy()
            .replace('\\', "/");

        let head = args.head.as_deref().unwrap_or("HEAD");
        let new_content = if args.head.is_some() {
            match git_show_file(&git_root, head, &rel) {
                Some(content) => content,
                None => {
                    if !git_ref_exists(&git_root, head) {
                        eprintln!("warning: could not retrieve {} at ref {}", rel, head);
                        continue;
                    }
                    // Ref is valid but file deleted at head — report all packages as removed
                    String::new()
                }
            }
        } else {
            std::fs::read_to_string(lf_path).unwrap_or_default()
        };
        // When --head is supplied, the file was detected from the head commit and may not exist
        // at base (newly added lockfile) — treat a missing base file as empty baseline.
        // Without --head (disk-based), a missing base means an invalid ref; warn and skip.
        let old_content = if args.head.is_some() {
            git_show_file(&git_root, &args.base, &rel).unwrap_or_default()
        } else {
            match git_show_file(&git_root, &args.base, &rel) {
                Some(content) => content,
                None => {
                    eprintln!("warning: could not retrieve {} at {}", rel, args.base);
                    continue;
                }
            }
        };

        let old_pkgs = lockfile::parse_lockfile(manager, &old_content);
        let new_pkgs = lockfile::parse_lockfile(manager, &new_content);
        let diff = lockfile::diff(&old_pkgs, &new_pkgs);

        outputs.push(LockfileDiffOutput {
            lockfile: rel,
            manager: manager_name(manager).to_string(),
            added: diff.added,
            removed: diff.removed,
            changed: diff.changed,
        });
    }

    if args.format == "paths" {
        for o in &outputs {
            for pkg in o
                .added
                .iter()
                .chain(o.removed.iter())
                .chain(o.changed.iter())
            {
                println!("{}", pkg);
            }
        }
    } else {
        println!("{}", serde_json::to_string_pretty(&outputs)?);
    }

    Ok(ExitCode::SUCCESS)
}

fn detect_lockfiles_from_head(git_root: &Path, head: &str, root: &Path) -> Vec<PathBuf> {
    let candidates = [
        "pnpm-lock.yaml",
        "package-lock.json",
        "npm-shrinkwrap.json",
        "yarn.lock",
        "bun.lock",
    ];
    let prefix = root
        .strip_prefix(git_root)
        .unwrap_or(std::path::Path::new(""))
        .to_string_lossy()
        .replace('\\', "/");
    candidates
        .iter()
        .filter(|name| {
            let rel = if prefix.is_empty() {
                name.to_string()
            } else {
                format!("{}/{}", prefix, name)
            };
            git_show_file(git_root, head, &rel).is_some()
        })
        .map(|name| root.join(name))
        .collect()
}

fn detect_lockfiles_in_root(root: &Path) -> Vec<PathBuf> {
    let candidates = [
        "pnpm-lock.yaml",
        "package-lock.json",
        "npm-shrinkwrap.json",
        "yarn.lock",
        "bun.lock",
    ];
    candidates
        .iter()
        .map(|name| root.join(name))
        .filter(|p| p.exists())
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

fn git_ref_exists(root: &Path, git_ref: &str) -> bool {
    std::process::Command::new("git")
        .args(["rev-parse", "--verify", git_ref])
        .current_dir(root)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
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

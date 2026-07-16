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
    let git_root = find_git_root(&root)?.unwrap_or_else(|| root.clone());

    let lockfile_paths = if let Some(lf) = args.lockfile {
        vec![root.join(lf)]
    } else if let Some(head) = args.head.as_deref() {
        // When --head is supplied, detect from the head commit so we find lockfiles
        // added or removed between base and head that don't exist on disk.
        if !git_ref_exists(&git_root, head)? {
            eprintln!(
                "warning: head ref `{}` does not exist; no lockfiles detected",
                head
            );
            return Ok(ExitCode::SUCCESS);
        }
        detect_lockfiles_from_head(&git_root, head, &root)?
    } else {
        let visible_paths = no_mistakes::codebase::ts_source::try_discover_visible_paths(&root)?;
        detect_lockfiles_in_root(&root, &visible_paths)
    };

    let mut outputs = Vec::new();

    for lf_path in &lockfile_paths {
        let basename = lf_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let Some(manager) = lockfile::detect_manager(basename) else {
            if lockfile::is_binary_lockfile(basename) {
                eprintln!("warning: `{basename}` is a binary lockfile and cannot be parsed for dependency changes");
            }
            continue;
        };
        let rel = lf_path
            .strip_prefix(&git_root)
            .unwrap_or(lf_path)
            .to_string_lossy()
            .replace('\\', "/");

        let head = args.head.as_deref().unwrap_or("HEAD");
        let new_content = if args.head.is_some() {
            let Some(c) = git_content_or_empty(&git_root, head, &rel)? else {
                eprintln!("warning: could not retrieve {} at ref {}", rel, head);
                continue;
            };
            c
        } else {
            std::fs::read_to_string(lf_path).unwrap_or_default()
        };
        let Some(old_content) = git_content_or_empty(&git_root, &args.base, &rel)? else {
            eprintln!("warning: could not retrieve {} at ref {}", rel, args.base);
            continue;
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

include!("lockfile/git.rs");

fn manager_name(m: PackageManager) -> &'static str {
    match m {
        PackageManager::Npm => "npm",
        PackageManager::Pnpm => "pnpm",
        PackageManager::Yarn => "yarn",
        PackageManager::Bun => "bun",
    }
}

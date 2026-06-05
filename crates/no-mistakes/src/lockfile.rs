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

    let lockfile_paths = if let Some(lf) = args.lockfile {
        vec![root.join(lf)]
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
            .strip_prefix(&root)
            .unwrap_or(lf_path)
            .to_string_lossy()
            .replace('\\', "/");

        let new_content = std::fs::read_to_string(lf_path).unwrap_or_default();
        let Some(old_content) = git_show_file(&root, &args.base, &rel) else {
            eprintln!("warning: could not retrieve {} at {}", rel, args.base);
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

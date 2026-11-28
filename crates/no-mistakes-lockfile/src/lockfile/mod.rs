pub mod bun;
mod ecosystem;
#[doc(hidden)]
pub mod keyed_diff;
pub mod npm;
pub mod pnpm;
#[doc(hidden)]
pub mod pnpm_alias;
mod revisions;
pub mod validate;
pub mod yarn;

pub use pnpm::{parse_importers, PnpmImporter, PnpmImporterDependency};
pub use revisions::{detect_lockfiles_at_refs, git_lockfile_path, git_ref_exists, git_show_file};
pub use validate::{packages_for_diff, packages_for_optional_diff};

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResolutionKind {
    Registry,
    Workspace,
    Tarball,
    Git,
    Directory,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedPackage {
    pub name: String,
    pub version: String,
    pub fingerprint: String,
    pub kind: ResolutionKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LockfileDiff {
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub changed: Vec<String>,
}

impl LockfileDiff {
    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.removed.is_empty() && self.changed.is_empty()
    }

    pub fn all_changed_names(&self) -> impl Iterator<Item = &str> {
        self.added
            .iter()
            .chain(self.removed.iter())
            .chain(self.changed.iter())
            .map(|s| s.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageManager {
    Npm,
    Pnpm,
    Yarn,
    Bun,
    Poetry,
    Uv,
    Pipfile,
    Go,
    Cargo,
    Gemfile,
    Composer,
}

impl PackageManager {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Npm => "npm",
            Self::Pnpm => "pnpm",
            Self::Yarn => "yarn",
            Self::Bun => "bun",
            Self::Poetry => "poetry",
            Self::Uv => "uv",
            Self::Pipfile => "pipfile",
            Self::Go => "go",
            Self::Cargo => "cargo",
            Self::Gemfile => "gemfile",
            Self::Composer => "composer",
        }
    }
}

pub const LOCKFILE_BASENAMES: &[&str] = &[
    "pnpm-lock.yaml",
    "package-lock.json",
    "npm-shrinkwrap.json",
    "yarn.lock",
    "bun.lock",
    "poetry.lock",
    "uv.lock",
    "Pipfile.lock",
    "go.mod",
    "Cargo.lock",
    "Gemfile.lock",
    "composer.lock",
];

pub fn detect_manager(basename: &str) -> Option<PackageManager> {
    match basename {
        "package-lock.json" | "npm-shrinkwrap.json" => Some(PackageManager::Npm),
        "pnpm-lock.yaml" => Some(PackageManager::Pnpm),
        "yarn.lock" => Some(PackageManager::Yarn),
        "bun.lock" => Some(PackageManager::Bun),
        "poetry.lock" => Some(PackageManager::Poetry),
        "uv.lock" => Some(PackageManager::Uv),
        "Pipfile.lock" => Some(PackageManager::Pipfile),
        "go.mod" => Some(PackageManager::Go),
        "Cargo.lock" => Some(PackageManager::Cargo),
        "Gemfile.lock" => Some(PackageManager::Gemfile),
        "composer.lock" => Some(PackageManager::Composer),
        _ => None,
    }
}

pub fn detect_planning_manager(basename: &str) -> Option<PackageManager> {
    detect_manager(basename).filter(|manager| {
        matches!(
            manager,
            PackageManager::Npm | PackageManager::Pnpm | PackageManager::Yarn | PackageManager::Bun
        )
    })
}

pub fn is_binary_lockfile(basename: &str) -> bool {
    basename == "bun.lockb"
}

pub fn binary_lockfile_warning(basename: &str) -> String {
    format!("`{basename}` is a binary lockfile and cannot be parsed for dependency changes")
}

pub fn read_worktree_lockfile(path: &std::path::Path) -> std::io::Result<Option<String>> {
    match std::fs::read_to_string(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        result => result.map(Some),
    }
}

/// Legacy best-effort extraction. Use `packages_for_diff` to distinguish invalid input.
pub fn parse_lockfile(manager: PackageManager, content: &str) -> Vec<ResolvedPackage> {
    match manager {
        PackageManager::Npm => npm::parse(content),
        PackageManager::Pnpm => pnpm::parse(content),
        PackageManager::Yarn => yarn::parse(content),
        PackageManager::Bun => bun::parse(content),
        PackageManager::Poetry => ecosystem::parse_toml(ecosystem::TomlKind::Poetry, content),
        PackageManager::Uv => ecosystem::parse_toml(ecosystem::TomlKind::Uv, content),
        PackageManager::Pipfile => ecosystem::parse_pipfile(content),
        PackageManager::Go => ecosystem::parse_gomod(content),
        PackageManager::Cargo => ecosystem::parse_toml(ecosystem::TomlKind::Cargo, content),
        PackageManager::Gemfile => ecosystem::parse_gemfile(content),
        PackageManager::Composer => ecosystem::parse_composer(content),
    }
}

pub fn diff(old: &[ResolvedPackage], new: &[ResolvedPackage]) -> LockfileDiff {
    let changes = keyed_diff::diff_packages(old, new);
    LockfileDiff {
        added: changes.added,
        removed: changes.removed,
        changed: changes.changed,
    }
}

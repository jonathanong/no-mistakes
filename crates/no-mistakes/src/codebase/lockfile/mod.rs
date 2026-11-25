pub mod bun;
pub mod npm;
pub mod pnpm;
pub(crate) mod pnpm_alias;
pub mod validate;
pub mod yarn;

pub use pnpm::{parse_importers, PnpmImporter, PnpmImporterDependency};

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
}

pub fn detect_manager(basename: &str) -> Option<PackageManager> {
    match basename {
        "package-lock.json" | "npm-shrinkwrap.json" => Some(PackageManager::Npm),
        "pnpm-lock.yaml" => Some(PackageManager::Pnpm),
        "yarn.lock" => Some(PackageManager::Yarn),
        "bun.lock" => Some(PackageManager::Bun),
        _ => None,
    }
}

pub fn is_binary_lockfile(basename: &str) -> bool {
    basename == "bun.lockb"
}

pub fn parse_lockfile(manager: PackageManager, content: &str) -> Vec<ResolvedPackage> {
    match manager {
        PackageManager::Npm => npm::parse(content),
        PackageManager::Pnpm => pnpm::parse(content),
        PackageManager::Yarn => yarn::parse(content),
        PackageManager::Bun => bun::parse(content),
    }
}

pub fn diff(old: &[ResolvedPackage], new: &[ResolvedPackage]) -> LockfileDiff {
    let changes = super::keyed_diff::diff_packages(old, new);
    LockfileDiff {
        added: changes.added,
        removed: changes.removed,
        changed: changes.changed,
    }
}

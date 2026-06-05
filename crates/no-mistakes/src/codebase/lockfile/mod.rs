pub mod bun;
pub mod npm;
pub mod pnpm;
pub mod yarn;

#[cfg(test)]
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
    use std::collections::{HashMap, HashSet};

    let mut old_map: HashMap<&str, HashSet<(&str, &str, &ResolutionKind)>> = HashMap::new();
    for p in old {
        old_map
            .entry(&p.name)
            .or_default()
            .insert((&p.version, &p.fingerprint, &p.kind));
    }
    let mut new_map: HashMap<&str, HashSet<(&str, &str, &ResolutionKind)>> = HashMap::new();
    for p in new {
        new_map
            .entry(&p.name)
            .or_default()
            .insert((&p.version, &p.fingerprint, &p.kind));
    }

    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut changed = Vec::new();

    for (name, old_set) in &old_map {
        match new_map.get(name) {
            None => removed.push((*name).to_string()),
            Some(new_set) if old_set != new_set => changed.push((*name).to_string()),
            _ => {}
        }
    }
    for name in new_map.keys() {
        if !old_map.contains_key(name) {
            added.push((*name).to_string());
        }
    }

    added.sort();
    removed.sort();
    changed.sort();

    LockfileDiff {
        added,
        removed,
        changed,
    }
}

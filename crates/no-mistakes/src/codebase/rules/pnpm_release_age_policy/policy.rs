use super::lockfile::lock_key_matches_selector;
use super::Options;
use std::collections::HashSet;

mod checks;
mod validation;

#[cfg(test)]
pub(super) use validation::{is_canonical_timestamp, is_exact_selector};

pub(super) struct Issue {
    pub(super) file: FileKind,
    pub(super) message: String,
    pub(super) target: String,
}

pub(super) enum FileKind {
    Workspace,
    Dependabot,
    Lockfile,
}

pub(super) struct Snapshot {
    pub(super) exclude: Vec<ExcludeEntry>,
    pub(super) cooldown: Option<Vec<CooldownEntry>>,
    pub(super) active_names: HashSet<String>,
    pub(super) lockfile_keys: Option<Vec<String>>,
}

pub(super) enum ExcludeEntry {
    Name(String),
    Other,
}

pub(super) enum CooldownEntry {
    Pattern(String),
    Other,
}

pub(super) fn check(opts: &Options, snapshot: &Snapshot) -> Vec<Issue> {
    let mut issues = Vec::new();
    let permanent = package_names(&opts.permanent_packages);
    let temporary = validation::temporary_selectors(opts, &mut issues);
    let registry: HashSet<String> = permanent.union(&temporary).cloned().collect();
    checks::exclude(&mut issues, snapshot, &registry, &permanent, &temporary);
    if let Some(cooldown) = &snapshot.cooldown {
        checks::dependabot(&mut issues, cooldown, &permanent);
    }
    checks::graph(&mut issues, opts, snapshot, &permanent);
    if let Some(keys) = &snapshot.lockfile_keys {
        checks::temporary_lockfile(&mut issues, keys, &temporary, lock_key_matches_selector);
    }
    issues
}

fn package_names(packages: &[super::PermanentPackage]) -> HashSet<String> {
    packages
        .iter()
        .filter(|package| !package.name.trim().is_empty())
        .map(|package| package.name.clone())
        .collect()
}

pub(super) fn push(issues: &mut Vec<Issue>, file: FileKind, target: &str, detail: &str) {
    issues.push(Issue {
        file,
        message: format!("\"{target}\" {detail}"),
        target: target.to_string(),
    });
}

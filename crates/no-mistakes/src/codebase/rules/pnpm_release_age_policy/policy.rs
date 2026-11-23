use super::lockfile::{lock_key_matches_selector, package_name_from_lock_key};
use super::Options;
use std::collections::HashSet;

mod checks;
mod package_glob;
mod registry;
pub(super) mod validation;

pub(super) struct Issue {
    pub(super) file: FileKind,
    pub(super) message: String,
    pub(super) target: String,
}

#[derive(Clone, Copy)]
pub(super) enum FileKind {
    Workspace,
    Dependabot,
    Lockfile,
}

pub(super) struct Snapshot {
    pub(super) exclude: Vec<ExcludeEntry>,
    pub(super) cooldown: Option<Vec<CooldownEntry>>,
    pub(super) manifest_names: HashSet<String>,
    pub(super) lockfile_keys: Option<Vec<String>>,
}

pub(super) enum ExcludeEntry {
    Name(String),
    Other,
    Conflict,
}

pub(super) enum CooldownEntry {
    Pattern(String),
    Other,
}

pub(super) fn check_with_complete_names(
    opts: &Options,
    snapshot: &Snapshot,
    complete: bool,
) -> Vec<Issue> {
    let mut issues = Vec::new();
    let lock_names: HashSet<&str> = snapshot
        .lockfile_keys
        .iter()
        .flatten()
        .filter_map(|key| package_name_from_lock_key(key))
        .collect();
    let active_names = snapshot
        .manifest_names
        .iter()
        .map(String::as_str)
        .chain(lock_names.iter().copied())
        .collect();
    let registry = registry::Registry::new(opts, &active_names, lock_names, complete, &mut issues);
    checks::exclude(&mut issues, &snapshot.exclude, &registry);
    if let Some(cooldown) = &snapshot.cooldown {
        checks::dependabot(&mut issues, cooldown, &registry);
    }
    checks::graph(
        &mut issues,
        opts,
        &active_names,
        &registry.permanent,
        complete,
    );
    if let Some(keys) = &snapshot.lockfile_keys {
        checks::temporary_lockfile(
            &mut issues,
            keys,
            &registry.temporary,
            lock_key_matches_selector,
        );
    }
    issues
}

pub(super) fn push(issues: &mut Vec<Issue>, file: FileKind, target: &str, detail: &str) {
    issues.push(Issue {
        file,
        message: format!("\"{target}\" {detail}"),
        target: target.to_string(),
    });
}

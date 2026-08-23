use super::lockfile::lock_key_matches_selector;
use super::Options;
use globset::GlobBuilder;
use std::collections::HashSet;

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
    let mut permanent = HashSet::new();
    for pkg in &opts.permanent_packages {
        if pkg.name.trim().is_empty() {
            continue;
        }
        permanent.insert(pkg.name.clone());
    }
    let temporary: HashSet<String> = opts.temporary_selectors.iter().cloned().collect();
    let registry: HashSet<String> = permanent.union(&temporary).cloned().collect();
    check_exclude(&mut issues, snapshot, &registry, &permanent, &temporary);
    if let Some(cooldown) = &snapshot.cooldown {
        check_dependabot(&mut issues, cooldown, &permanent);
    }
    check_graph(&mut issues, opts, snapshot, &permanent);
    if let Some(keys) = &snapshot.lockfile_keys {
        check_temporary_lockfile(&mut issues, keys, &temporary);
    }
    issues
}

fn check_exclude(
    issues: &mut Vec<Issue>,
    snapshot: &Snapshot,
    registry: &HashSet<String>,
    permanent: &HashSet<String>,
    temporary: &HashSet<String>,
) {
    let mut seen = HashSet::new();
    let mut yaml_names = HashSet::new();
    for entry in &snapshot.exclude {
        let ExcludeEntry::Name(name) = entry else {
            continue;
        };
        if !seen.insert(name.clone()) {
            push(issues, FileKind::Workspace, name, "duplicates");
        }
        yaml_names.insert(name.clone());
        if !registry.contains(name) {
            push(
                issues,
                FileKind::Workspace,
                name,
                "is not in a release-age exemption registry",
            );
        }
    }
    for name in permanent {
        if !yaml_names.contains(name) {
            push(
                issues,
                FileKind::Workspace,
                name,
                "is missing from minimumReleaseAgeExclude",
            );
        }
    }
    for selector in temporary {
        if !yaml_names.contains(selector) {
            push(
                issues,
                FileKind::Workspace,
                selector,
                "is missing from minimumReleaseAgeExclude",
            );
        }
    }
}

fn check_dependabot(
    issues: &mut Vec<Issue>,
    cooldown: &[CooldownEntry],
    permanent: &HashSet<String>,
) {
    let mut patterns = Vec::new();
    for entry in cooldown {
        match entry {
            CooldownEntry::Pattern(pattern) => patterns.push(pattern.as_str()),
            CooldownEntry::Other => push(
                issues,
                FileKind::Dependabot,
                "cooldown.exclude",
                "must be a string glob pattern",
            ),
        }
    }
    for name in permanent {
        if !patterns.iter().any(|pattern| glob_matches(pattern, name)) {
            push(
                issues,
                FileKind::Dependabot,
                name,
                "is not covered by npm cooldown.exclude",
            );
        }
    }
}

fn check_graph(
    issues: &mut Vec<Issue>,
    opts: &Options,
    snapshot: &Snapshot,
    permanent: &HashSet<String>,
) {
    if !opts.scoped_prefixes.is_empty() {
        for name in &snapshot.active_names {
            if opts
                .scoped_prefixes
                .iter()
                .any(|prefix| name.starts_with(prefix))
                && !permanent.contains(name)
            {
                push(
                    issues,
                    FileKind::Lockfile,
                    name,
                    "is an active first-party package missing from permanentPackages",
                );
            }
        }
    }
    for name in permanent {
        if !snapshot.active_names.contains(name) {
            push(
                issues,
                FileKind::Lockfile,
                name,
                "is registered but absent from package manifests and the lockfile",
            );
        }
    }
}

fn check_temporary_lockfile(issues: &mut Vec<Issue>, keys: &[String], temporary: &HashSet<String>) {
    for selector in temporary {
        if !keys
            .iter()
            .any(|key| lock_key_matches_selector(key, selector))
        {
            push(
                issues,
                FileKind::Lockfile,
                selector,
                "is absent from lockfile packages",
            );
        }
    }
}

fn glob_matches(pattern: &str, name: &str) -> bool {
    GlobBuilder::new(pattern)
        .literal_separator(false)
        .build()
        .ok()
        .is_some_and(|glob| glob.compile_matcher().is_match(name))
}

fn push(issues: &mut Vec<Issue>, file: FileKind, target: &str, detail: &str) {
    issues.push(Issue {
        file,
        message: format!("\"{target}\" {detail}"),
        target: target.to_string(),
    });
}

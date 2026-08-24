use super::{push, CooldownEntry, ExcludeEntry, FileKind, Issue, Snapshot};
use crate::codebase::rules::pnpm_release_age_policy::Options;
use globset::GlobBuilder;
use std::collections::HashSet;

pub(super) fn exclude(
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
    report_missing(issues, permanent, &yaml_names);
    report_missing(issues, temporary, &yaml_names);
}

fn report_missing(issues: &mut Vec<Issue>, wanted: &HashSet<String>, yaml_names: &HashSet<String>) {
    for name in wanted {
        if !yaml_names.contains(name) {
            push(
                issues,
                FileKind::Workspace,
                name,
                "is missing from minimumReleaseAgeExclude",
            );
        }
    }
}

pub(super) fn dependabot(
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

pub(super) fn graph(
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

pub(super) fn temporary_lockfile(
    issues: &mut Vec<Issue>,
    keys: &[String],
    temporary: &HashSet<String>,
    matches_selector: fn(&str, &str) -> bool,
) {
    for selector in temporary {
        if !keys.iter().any(|key| matches_selector(key, selector)) {
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

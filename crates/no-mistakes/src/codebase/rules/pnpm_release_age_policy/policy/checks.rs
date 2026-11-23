use super::package_glob::PackageGlob;
use super::registry::{invalid_glob, Registry};
use super::{push, CooldownEntry, ExcludeEntry, FileKind, Issue, Options};
use std::collections::HashSet;

#[derive(Default)]
struct Exemptions<'a> {
    exact: HashSet<&'a str>,
    globs: Vec<PackageGlob>,
}

impl<'a> Exemptions<'a> {
    fn insert(
        &mut self,
        issues: &mut Vec<Issue>,
        file: FileKind,
        value: &'a str,
        registry: &Registry,
    ) {
        if PackageGlob::is_glob(value) {
            if let Some(glob) = PackageGlob::new(value) {
                registry.check_glob(issues, file, value, &glob);
                self.globs.push(glob);
            } else {
                invalid_glob(issues, file, value);
            }
        } else {
            self.exact.insert(value);
            if matches!(file, FileKind::Workspace)
                && !registry.permanent.contains(value)
                && !registry.temporary.contains(value)
            {
                push(
                    issues,
                    file,
                    value,
                    "is not in a release-age exemption registry",
                );
            }
        }
    }

    fn covers(&self, name: &str) -> bool {
        self.exact.contains(name) || self.globs.iter().any(|glob| glob.matches(name))
    }
}

pub(super) fn exclude(issues: &mut Vec<Issue>, entries: &[ExcludeEntry], registry: &Registry) {
    if entries
        .iter()
        .any(|entry| matches!(entry, ExcludeEntry::Conflict))
    {
        return;
    }
    let mut seen = HashSet::new();
    let mut exemptions = Exemptions::default();
    for entry in entries {
        let ExcludeEntry::Name(name) = entry else {
            continue;
        };
        if !seen.insert(name) {
            push(issues, FileKind::Workspace, name, "duplicates");
        }
        exemptions.insert(issues, FileKind::Workspace, name, registry);
    }
    for name in registry
        .permanent
        .iter()
        .copied()
        .filter(|name| !exemptions.covers(name))
        .chain(
            registry
                .temporary
                .iter()
                .map(String::as_str)
                .filter(|selector| !exemptions.exact.contains(selector)),
        )
    {
        push(
            issues,
            FileKind::Workspace,
            name,
            "is missing from minimumReleaseAgeExclude",
        );
    }
}

pub(super) fn dependabot(issues: &mut Vec<Issue>, cooldown: &[CooldownEntry], registry: &Registry) {
    let mut exemptions = Exemptions::default();
    for entry in cooldown {
        match entry {
            CooldownEntry::Pattern(pattern) => {
                exemptions.insert(issues, FileKind::Dependabot, pattern, registry);
            }
            CooldownEntry::Other => push(
                issues,
                FileKind::Dependabot,
                "cooldown.exclude",
                "must be a string glob pattern",
            ),
        }
    }
    for name in &registry.permanent {
        if !exemptions.covers(name) {
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
    active_names: &HashSet<&str>,
    permanent: &HashSet<&str>,
    complete: bool,
) {
    if !opts.scoped_prefixes.is_empty() {
        for name in active_names {
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
        if complete && !active_names.contains(name) {
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

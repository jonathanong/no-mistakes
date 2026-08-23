use super::lockfile::lock_key_matches_selector;
use super::{Options, TemporaryGroup};
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
    let temporary = validated_temporary_selectors(opts, &mut issues);
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

fn validated_temporary_selectors(opts: &Options, issues: &mut Vec<Issue>) -> HashSet<String> {
    let mut temporary = HashSet::new();
    for selector in &opts.temporary_selectors {
        insert_temporary_selector(&mut temporary, issues, selector);
    }
    for group in &opts.temporary_groups {
        if !valid_group(group, issues) {
            continue;
        }
        for selector in &group.selectors {
            insert_temporary_selector(&mut temporary, issues, selector);
        }
    }
    temporary
}

fn valid_group(group: &TemporaryGroup, issues: &mut Vec<Issue>) -> bool {
    let mut valid = true;
    if group.selectors.is_empty() {
        push(
            issues,
            FileKind::Workspace,
            "temporaryGroups",
            "selectors must contain at least one exact package@version selector",
        );
        valid = false;
    }
    if group.reason.trim().is_empty() {
        push(
            issues,
            FileKind::Workspace,
            "temporaryGroups",
            "reason must be non-empty",
        );
        valid = false;
    }
    if !is_canonical_timestamp(&group.eligible_for_removal_at) {
        push(
            issues,
            FileKind::Workspace,
            "temporaryGroups",
            "eligibleForRemovalAt must be canonical YYYY-MM-DDTHH:mm:ssZ",
        );
        valid = false;
    }
    for selector in &group.selectors {
        if !is_exact_selector(selector) {
            push(
                issues,
                FileKind::Workspace,
                selector,
                "must be an exact package@version selector",
            );
            valid = false;
        }
    }
    valid
}

fn insert_temporary_selector(
    temporary: &mut HashSet<String>,
    issues: &mut Vec<Issue>,
    selector: &str,
) {
    if !temporary.insert(selector.to_string()) {
        push(
            issues,
            FileKind::Workspace,
            selector,
            "duplicates another temporary selector",
        );
    }
}

pub(super) fn is_exact_selector(selector: &str) -> bool {
    let Some((name, version)) = selector.rsplit_once('@') else {
        return false;
    };
    is_package_name(name)
        && version.split(['.', '-', '+']).all(|part| {
            !part.is_empty()
                && part
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric())
        })
        && version.chars().any(|character| character.is_ascii_digit())
}

fn is_package_name(name: &str) -> bool {
    if let Some(scoped) = name.strip_prefix('@') {
        let Some((scope, package)) = scoped.split_once('/') else {
            return false;
        };
        !scope.is_empty() && !package.is_empty() && !package.contains('@')
    } else {
        !name.is_empty() && !name.contains('@')
    }
}

pub(super) fn is_canonical_timestamp(timestamp: &str) -> bool {
    let bytes = timestamp.as_bytes();
    if bytes.len() != 20
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes[10] != b'T'
        || bytes[13] != b':'
        || bytes[16] != b':'
        || bytes[19] != b'Z'
        || [0, 1, 2, 3, 5, 6, 8, 9, 11, 12, 14, 15, 17, 18]
            .into_iter()
            .any(|index| !bytes[index].is_ascii_digit())
    {
        return false;
    }
    let number = |start| {
        std::str::from_utf8(&bytes[start..start + 2])
            .unwrap()
            .parse::<u32>()
            .unwrap()
    };
    let year = std::str::from_utf8(&bytes[..4])
        .unwrap()
        .parse::<u32>()
        .unwrap();
    let month = number(5);
    let day = number(8);
    let max_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        _ => return false,
    };
    (1..=max_day).contains(&day) && number(11) < 24 && number(14) < 60 && number(17) < 60
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

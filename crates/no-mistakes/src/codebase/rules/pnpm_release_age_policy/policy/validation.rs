use super::{push, FileKind, Issue};
use crate::codebase::rules::pnpm_release_age_policy::{Options, TemporaryGroup};
use std::collections::HashSet;

pub(super) fn temporary_selectors(opts: &Options, issues: &mut Vec<Issue>) -> HashSet<String> {
    let mut temporary = HashSet::new();
    let mut seen = HashSet::new();
    for selector in &opts.temporary_selectors {
        record_selector(&mut seen, issues, selector);
        temporary.insert(selector.to_string());
    }
    for group in &opts.temporary_groups {
        let valid = valid_group(group, issues);
        for selector in &group.selectors {
            record_selector(&mut seen, issues, selector);
            if valid {
                temporary.insert(selector.to_string());
            }
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

fn record_selector(seen: &mut HashSet<String>, issues: &mut Vec<Issue>, selector: &str) {
    if !seen.insert(selector.to_string()) {
        push(
            issues,
            FileKind::Workspace,
            selector,
            "duplicates another temporary selector",
        );
    }
}

pub(crate) fn is_exact_selector(selector: &str) -> bool {
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

pub(crate) fn is_canonical_timestamp(timestamp: &str) -> bool {
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
    let max_day = match number(5) {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        _ => return false,
    };
    (1..=max_day).contains(&number(8)) && number(11) < 24 && number(14) < 60 && number(17) < 60
}

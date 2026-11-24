use super::{diff_entries, KeyChanges};
use crate::codebase::lockfile::{self, ResolutionKind, ResolvedPackage};
use anyhow::Result;
use std::collections::{BTreeMap, HashSet};
use std::hash::Hash;

type Packages = BTreeMap<String, HashSet<(String, String, ResolutionKind)>>;

pub(super) fn pnpm_entries(source: &str) -> Result<Packages> {
    let packages = lockfile::pnpm::parse_checked(source)
        .map_err(|error| anyhow::anyhow!("invalid pnpm lockfile: {error:?}"))?;
    Ok(group_owned(packages))
}

pub(super) fn npm_entries(source: &str) -> Result<Packages> {
    Ok(group_owned(lockfile::npm::parse_checked(source)?))
}

fn group_owned(packages: Vec<ResolvedPackage>) -> Packages {
    group(packages.into_iter().map(|package| {
        (
            package.name,
            (package.version, package.fingerprint, package.kind),
        )
    }))
}

pub(in crate::codebase) fn diff_packages(
    old: &[ResolvedPackage],
    new: &[ResolvedPackage],
) -> KeyChanges {
    diff_entries(&group(package_records(old)), &group(package_records(new)))
}

fn package_records(
    packages: &[ResolvedPackage],
) -> impl Iterator<Item = (&str, (&str, &str, &ResolutionKind))> {
    packages.iter().map(|package| {
        (
            package.name.as_str(),
            (
                package.version.as_str(),
                package.fingerprint.as_str(),
                &package.kind,
            ),
        )
    })
}

fn group<K: Ord, V: Eq + Hash>(records: impl Iterator<Item = (K, V)>) -> BTreeMap<K, HashSet<V>> {
    let mut entries = BTreeMap::<_, HashSet<_>>::new();
    for (key, value) in records {
        entries.entry(key).or_default().insert(value);
    }
    entries
}

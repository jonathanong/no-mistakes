use super::{npm, pnpm, ResolutionKind, ResolvedPackage};
use crate::keyed_diff::{diff_entries, diff_parsed, KeyChanges, KeyFormat};
use anyhow::Result;
use std::collections::{BTreeMap, HashSet};
use std::hash::Hash;

type Packages = BTreeMap<String, HashSet<(String, String, ResolutionKind)>>;

#[doc(hidden)]
pub fn diff_pnpm_keys(
    old: Option<&[u8]>,
    new: Option<&[u8]>,
) -> Result<(KeyChanges, [Vec<String>; 2])> {
    diff_parsed(KeyFormat::Pnpm, old, new, pnpm_entries)
}

#[doc(hidden)]
pub fn diff_npm_keys(
    old: Option<&[u8]>,
    new: Option<&[u8]>,
) -> Result<(KeyChanges, [Vec<String>; 2])> {
    diff_parsed(KeyFormat::Npm, old, new, npm_entries)
}

fn pnpm_entries(source: &str) -> Result<Packages> {
    let packages = pnpm::parse_checked(source)
        .map_err(|error| anyhow::anyhow!("invalid pnpm lockfile: {error:?}"))?;
    Ok(group_owned(packages))
}

fn npm_entries(source: &str) -> Result<Packages> {
    Ok(group_owned(npm::parse_checked(source)?))
}

fn group_owned(packages: Vec<ResolvedPackage>) -> Packages {
    group(packages.into_iter().map(|package| {
        (
            package.name,
            (package.version, package.fingerprint, package.kind),
        )
    }))
}

pub(super) fn diff_packages(old: &[ResolvedPackage], new: &[ResolvedPackage]) -> KeyChanges {
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

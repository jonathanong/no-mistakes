use super::super::super::{ResolutionKind, ResolvedPackage};
use super::TomlError;
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Deserialize)]
struct PoetryLock {
    #[serde(default)]
    package: Vec<PoetryPackage>,
    #[serde(default)]
    metadata: Option<PoetryMetadata>,
}

#[derive(Deserialize)]
struct PoetryPackage {
    name: String,
    version: String,
    #[serde(default)]
    files: Vec<PoetryFile>,
    #[serde(default)]
    source: Option<PoetrySource>,
}

#[derive(Deserialize)]
struct PoetryFile {
    #[serde(default)]
    hash: Option<String>,
}

#[derive(Deserialize, Default)]
struct PoetrySource {
    #[serde(default, rename = "type")]
    kind: Option<String>,
    #[serde(default)]
    reference: Option<String>,
    #[serde(default)]
    resolved_reference: Option<String>,
}

#[derive(Deserialize, Default)]
struct PoetryMetadata {
    #[serde(default)]
    files: BTreeMap<String, Vec<PoetryFile>>,
}

pub(super) fn parse(content: &str) -> Result<Vec<ResolvedPackage>, TomlError> {
    let lock: PoetryLock = toml::from_str(content).map_err(|_| TomlError)?;
    if lock.package.is_empty() && lock.metadata.is_none() {
        return Err(TomlError);
    }
    let mut packages: Vec<_> = lock.package.into_iter().map(package).collect();
    if let Some(metadata) = lock.metadata {
        fill_metadata(&mut packages, &metadata.files);
    }
    Ok(packages)
}

fn package(pkg: PoetryPackage) -> ResolvedPackage {
    let kind = kind_of(pkg.source.as_ref());
    ResolvedPackage {
        name: pkg.name,
        version: pkg.version,
        fingerprint: fingerprint(kind, pkg.source.as_ref(), &pkg.files),
        kind,
    }
}

fn kind_of(source: Option<&PoetrySource>) -> ResolutionKind {
    match source.and_then(|source| source.kind.as_deref()) {
        Some("git") => ResolutionKind::Git,
        Some("directory") => ResolutionKind::Directory,
        Some("url") => ResolutionKind::Tarball,
        _ => ResolutionKind::Registry,
    }
}

fn fingerprint(
    kind: ResolutionKind,
    source: Option<&PoetrySource>,
    files: &[PoetryFile],
) -> String {
    if kind == ResolutionKind::Git {
        if let Some(source) = source {
            if let Some(pin) = nonempty(&source.resolved_reference) {
                return pin;
            }
            if let Some(pin) = nonempty(&source.reference) {
                return pin;
            }
        }
    }
    first_hash(files)
}

fn fill_metadata(packages: &mut [ResolvedPackage], files: &BTreeMap<String, Vec<PoetryFile>>) {
    for package in packages {
        let Some(entries) = files.get(&package.name) else {
            continue;
        };
        if package.fingerprint.is_empty() {
            let hash = first_hash(entries);
            if !hash.is_empty() {
                package.fingerprint = hash;
            }
        }
    }
}

fn first_hash(files: &[PoetryFile]) -> String {
    files
        .iter()
        .find_map(|file| nonempty(&file.hash))
        .unwrap_or_default()
}

fn nonempty(value: &Option<String>) -> Option<String> {
    value
        .as_deref()
        .filter(|text| !text.is_empty())
        .map(str::to_string)
}

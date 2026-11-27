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
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    subdirectory: Option<String>,
}

#[derive(Deserialize, Default)]
struct PoetryMetadata {
    #[serde(default, rename = "lock-version")]
    lock_version: Option<String>,
    #[serde(default)]
    hashes: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    files: BTreeMap<String, Vec<PoetryFile>>,
}

pub(super) fn parse(content: &str) -> Result<Vec<ResolvedPackage>, TomlError> {
    let lock: PoetryLock = toml::from_str(content).map_err(|_| TomlError)?;
    let lock_version = lock
        .metadata
        .as_ref()
        .and_then(|metadata| metadata.lock_version.as_deref());
    if lock.package.is_empty()
        && !lock_version
            .and_then(|version| version.split_once('.'))
            .is_some_and(|(major, minor)| {
                major.parse::<u64>().is_ok() && minor.parse::<u64>().is_ok()
            })
    {
        return Err(TomlError);
    }
    let mut packages: Vec<_> = lock.package.into_iter().map(package).collect();
    if let Some(metadata) = lock.metadata {
        fill_metadata(&mut packages, &metadata);
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
        Some("url" | "file") => ResolutionKind::Tarball,
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
            let pin = nonempty(&source.resolved_reference).or_else(|| nonempty(&source.reference));
            if let Some(pin) = pin {
                return super::super::source_fingerprint(
                    &pin,
                    "subdirectory",
                    source.subdirectory.as_deref(),
                );
            }
        }
    }
    if kind == ResolutionKind::Directory {
        if let Some(path) = source.and_then(|source| nonempty(&source.url)) {
            return path;
        }
    }
    artifact_hashes(files)
}

fn fill_metadata(packages: &mut [ResolvedPackage], metadata: &PoetryMetadata) {
    for package in packages
        .iter_mut()
        .filter(|package| package.fingerprint.is_empty())
    {
        if let Some(hashes) = metadata.hashes.get(&package.name) {
            package.fingerprint = super::super::hash_fingerprint(hashes.iter().map(String::as_str));
        } else if let Some(files) = metadata.files.get(&package.name) {
            package.fingerprint = artifact_hashes(files);
        }
    }
}

fn artifact_hashes(files: &[PoetryFile]) -> String {
    super::super::hash_fingerprint(files.iter().filter_map(|file| file.hash.as_deref()))
}

fn nonempty(value: &Option<String>) -> Option<String> {
    value
        .as_deref()
        .filter(|text| !text.is_empty())
        .map(str::to_string)
}

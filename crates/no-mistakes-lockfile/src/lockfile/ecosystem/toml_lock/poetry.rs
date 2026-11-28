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
    file: Option<String>,
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
    Ok(lock
        .package
        .into_iter()
        .map(|pkg| package(pkg, lock.metadata.as_ref()))
        .collect())
}

fn package(pkg: PoetryPackage, metadata: Option<&PoetryMetadata>) -> ResolvedPackage {
    let source = pkg.source.as_ref();
    let kind = kind_of(source);
    let mut fingerprint = fingerprint(kind, source, &pkg.files);
    if fingerprint.is_empty() {
        if let Some(metadata) = metadata {
            fingerprint = legacy_fingerprint(&pkg.name, source, metadata);
        }
    }
    if fingerprint.is_empty() && kind == ResolutionKind::Tarball {
        if let Some(url) = source.and_then(|source| nonempty(&source.url)) {
            fingerprint = url;
        }
    }
    ResolvedPackage {
        name: pkg.name,
        version: pkg.version,
        fingerprint,
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
    artifact_hashes(files.iter())
}

fn legacy_fingerprint(
    name: &str,
    source: Option<&PoetrySource>,
    metadata: &PoetryMetadata,
) -> String {
    if let Some(hashes) = metadata.hashes.get(name) {
        return super::super::hash_fingerprint(hashes.iter().map(String::as_str));
    }
    let Some(files) = metadata.files.get(name) else {
        return String::new();
    };
    let archive = source
        .filter(|source| source.kind.as_deref() == Some("file"))
        .and_then(|source| source.url.as_deref())
        .and_then(|path| path.rsplit(['/', '\\']).next());
    artifact_hashes(
        files
            .iter()
            .filter(|file| archive.is_none_or(|name| file.file.as_deref() == Some(name))),
    )
}

fn artifact_hashes<'a>(files: impl Iterator<Item = &'a PoetryFile>) -> String {
    super::super::hash_fingerprint(files.filter_map(|file| file.hash.as_deref()))
}

fn nonempty(value: &Option<String>) -> Option<String> {
    value
        .as_deref()
        .filter(|text| !text.is_empty())
        .map(str::to_string)
}

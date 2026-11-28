use super::super::super::{ResolutionKind, ResolvedPackage};
use super::TomlError;
use serde::Deserialize;
use std::num::NonZeroU32;

#[derive(Deserialize)]
struct CargoLock {
    #[serde(default)]
    version: Option<NonZeroU32>,
    #[serde(default)]
    package: Vec<CargoPackage>,
    #[serde(default)]
    root: Option<CargoPackage>,
    #[serde(default)]
    metadata: toml::Table,
}

#[derive(Deserialize)]
struct CargoPackage {
    name: String,
    version: String,
    #[serde(default)]
    source: Option<String>,
    #[serde(default)]
    checksum: Option<String>,
}

pub(super) fn parse(content: &str) -> Result<Vec<ResolvedPackage>, TomlError> {
    let lock: CargoLock = toml::from_str(content).map_err(|_| TomlError)?;
    if lock.version.is_none() && lock.package.is_empty() && lock.root.is_none() {
        return Err(TomlError);
    }
    lock.package
        .into_iter()
        .chain(lock.root)
        .map(|pkg| package(pkg, &lock.metadata))
        .collect()
}

fn package(pkg: CargoPackage, metadata: &toml::Table) -> Result<ResolvedPackage, TomlError> {
    if !cargo_version(&pkg.version) {
        return Err(TomlError);
    }
    let checksum = pkg.checksum.as_deref().or_else(|| {
        let source = pkg.source.as_deref()?;
        let key = format!("checksum {} {} ({source})", pkg.name, pkg.version);
        metadata
            .get(&key)?
            .as_str()
            .filter(|value| *value != "<none>")
    });
    let (kind, fingerprint) = resolution(pkg.source.as_deref(), checksum.map(str::to_string));
    Ok(ResolvedPackage {
        name: pkg.name,
        version: pkg.version,
        fingerprint,
        kind,
    })
}

fn resolution(source: Option<&str>, checksum: Option<String>) -> (ResolutionKind, String) {
    let Some(source) = source else {
        return (ResolutionKind::Workspace, checksum.unwrap_or_default());
    };
    if source.starts_with("registry+") || source.starts_with("sparse+") {
        return (ResolutionKind::Registry, checksum.unwrap_or_default());
    }
    if source.starts_with("git+") {
        return (
            ResolutionKind::Git,
            checksum.unwrap_or_else(|| git_query_fingerprint(source)),
        );
    }
    if source.starts_with("path+") {
        return (ResolutionKind::Directory, checksum.unwrap_or_default());
    }
    (
        ResolutionKind::Other,
        checksum.unwrap_or_else(|| source.to_string()),
    )
}

fn cargo_version(version: &str) -> bool {
    let (version, build) = match version.split_once('+') {
        Some((version, build)) => (version, Some(build)),
        None => (version, None),
    };
    if build.is_some_and(|build| !dotted_idents(build, true)) {
        return false;
    }
    let (release, pre) = match version.split_once('-') {
        Some((release, pre)) => (release, Some(pre)),
        None => (version, None),
    };
    pre.is_none_or(|pre| dotted_idents(pre, false)) && release_triple(release)
}

fn release_triple(release: &str) -> bool {
    let mut parts = release.split('.');
    parts.next().is_some_and(numeric_component)
        && parts.next().is_some_and(numeric_component)
        && parts.next().is_some_and(numeric_component)
        && parts.next().is_none()
}

fn dotted_idents(value: &str, build: bool) -> bool {
    !value.is_empty() && value.split('.').all(|ident| ident_ok(ident, build))
}

fn ident_ok(ident: &str, build: bool) -> bool {
    let bytes = ident.as_bytes();
    !bytes.is_empty()
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'-')
        && (build || !bytes.iter().all(|byte| byte.is_ascii_digit()) || numeric_component(ident))
}

fn numeric_component(part: &str) -> bool {
    let bytes = part.as_bytes();
    !bytes.is_empty()
        && bytes.iter().all(|byte| byte.is_ascii_digit())
        && (bytes.len() == 1 || bytes[0] != b'0')
}

fn git_query_fingerprint(source: &str) -> String {
    let (head, fragment) = match source.split_once('#') {
        Some((head, fragment)) => (head, Some(fragment)),
        None => (source, None),
    };
    let Some((prefix, query)) = head.split_once('?') else {
        return source.to_string();
    };
    if !query.contains('%') {
        return source.to_string();
    }
    let Some(query) = percent_decode(query) else {
        return source.to_string();
    };
    match fragment {
        Some(fragment) => format!("{prefix}?{query}#{fragment}"),
        None => format!("{prefix}?{query}"),
    }
}

fn percent_decode(query: &str) -> Option<String> {
    let bytes = query.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] != b'%' {
            out.push(bytes[index]);
            index += 1;
            continue;
        }
        let hi = hex(*bytes.get(index + 1)?)?;
        let lo = hex(*bytes.get(index + 2)?)?;
        out.push((hi << 4) | lo);
        index += 3;
    }
    String::from_utf8(out).ok()
}

fn hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

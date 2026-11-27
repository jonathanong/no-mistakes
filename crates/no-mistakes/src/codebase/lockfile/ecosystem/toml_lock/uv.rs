use super::super::super::{ResolutionKind, ResolvedPackage};
use super::TomlError;
use serde::Deserialize;

#[derive(Deserialize)]
struct UvLock {
    #[serde(default)]
    version: Option<i64>,
    #[serde(default)]
    package: Vec<UvPackage>,
}

#[derive(Deserialize)]
struct UvPackage {
    name: String,
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    source: Option<UvSource>,
    #[serde(default)]
    wheels: Vec<UvDistribution>,
    #[serde(default)]
    sdist: Option<UvDistribution>,
}

#[derive(Deserialize, Default)]
struct UvSource {
    #[serde(default)]
    git: Option<String>,
    #[serde(default)]
    rev: Option<String>,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    editable: Option<String>,
    #[serde(default)]
    directory: Option<String>,
    #[serde(default, rename = "virtual")]
    r#virtual: Option<String>,
}

#[derive(Deserialize)]
struct UvDistribution {
    #[serde(default)]
    hash: Option<String>,
}

pub(super) fn parse(content: &str) -> Result<Vec<ResolvedPackage>, TomlError> {
    let lock: UvLock = toml::from_str(content).map_err(|_| TomlError)?;
    if lock.version.is_none() && lock.package.is_empty() {
        return Err(TomlError);
    }
    lock.package.into_iter().map(package).collect()
}

fn package(pkg: UvPackage) -> Result<ResolvedPackage, TomlError> {
    let distribution_hash = pkg
        .wheels
        .iter()
        .find_map(|wheel| wheel.hash.as_deref())
        .or_else(|| pkg.sdist.as_ref().and_then(|sdist| sdist.hash.as_deref()));
    let (kind, fingerprint) = resolution(pkg.source.as_ref(), distribution_hash);
    let version = match pkg.version {
        Some(version) => version,
        None if kind == ResolutionKind::Directory => String::new(),
        None => return Err(TomlError),
    };
    Ok(ResolvedPackage {
        name: pkg.name,
        version,
        fingerprint,
        kind,
    })
}

fn resolution(
    source: Option<&UvSource>,
    distribution_hash: Option<&str>,
) -> (ResolutionKind, String) {
    let Some(source) = source else {
        return (
            ResolutionKind::Registry,
            distribution_hash.unwrap_or_default().to_string(),
        );
    };
    if let Some(git) = source.git.as_deref() {
        return (ResolutionKind::Git, git_pin(git, source.rev.as_deref()));
    }
    if let Some(path) = source
        .editable
        .as_deref()
        .or(source.directory.as_deref())
        .or(source.r#virtual.as_deref())
    {
        return (ResolutionKind::Directory, path.to_string());
    }
    if let Some(archive) = source.url.as_deref().or(source.path.as_deref()) {
        return (
            ResolutionKind::Tarball,
            distribution_hash.unwrap_or(archive).to_string(),
        );
    }
    (
        ResolutionKind::Registry,
        distribution_hash.unwrap_or_default().to_string(),
    )
}

fn git_pin(url: &str, rev: Option<&str>) -> String {
    url.split_once('#')
        .map(|(_, fragment)| fragment)
        .filter(|fragment| !fragment.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| rev.unwrap_or("").to_string())
}

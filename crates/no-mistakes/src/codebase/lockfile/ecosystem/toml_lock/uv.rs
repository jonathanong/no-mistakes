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
    version: String,
    #[serde(default)]
    source: Option<UvSource>,
    #[serde(default)]
    wheels: Vec<UvWheel>,
}

#[derive(Deserialize, Default)]
struct UvSource {
    #[serde(default)]
    git: Option<String>,
    #[serde(default)]
    rev: Option<String>,
    #[serde(default)]
    editable: Option<toml::Value>,
    #[serde(default)]
    directory: Option<toml::Value>,
    #[serde(default, rename = "virtual")]
    r#virtual: Option<toml::Value>,
}

#[derive(Deserialize)]
struct UvWheel {
    #[serde(default)]
    hash: Option<String>,
}

pub(super) fn parse(content: &str) -> Result<Vec<ResolvedPackage>, TomlError> {
    let lock: UvLock = toml::from_str(content).map_err(|_| TomlError)?;
    if lock.version.is_none() && lock.package.is_empty() {
        return Err(TomlError);
    }
    Ok(lock.package.into_iter().map(package).collect())
}

fn package(pkg: UvPackage) -> ResolvedPackage {
    let (kind, fingerprint) = resolution(pkg.source.as_ref(), &pkg.wheels);
    ResolvedPackage {
        name: pkg.name,
        version: pkg.version,
        fingerprint,
        kind,
    }
}

fn resolution(source: Option<&UvSource>, wheels: &[UvWheel]) -> (ResolutionKind, String) {
    let Some(source) = source else {
        return (ResolutionKind::Registry, wheel_hash(wheels));
    };
    if let Some(git) = source.git.as_deref() {
        return (ResolutionKind::Git, git_pin(git, source.rev.as_deref()));
    }
    if source.editable.is_some() || source.directory.is_some() || source.r#virtual.is_some() {
        return (ResolutionKind::Directory, String::new());
    }
    (ResolutionKind::Registry, wheel_hash(wheels))
}

fn git_pin(url: &str, rev: Option<&str>) -> String {
    url.split_once('#')
        .map(|(_, fragment)| fragment)
        .filter(|fragment| !fragment.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| rev.unwrap_or("").to_string())
}

fn wheel_hash(wheels: &[UvWheel]) -> String {
    wheels
        .iter()
        .find_map(|wheel| wheel.hash.clone())
        .unwrap_or_default()
}

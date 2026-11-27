use super::super::super::{ResolutionKind, ResolvedPackage};
use super::TomlError;
use serde::Deserialize;

#[derive(Deserialize)]
struct CargoLock {
    #[serde(default)]
    version: Option<i64>,
    #[serde(default)]
    package: Vec<CargoPackage>,
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
    if lock.version.is_none() && lock.package.is_empty() {
        return Err(TomlError);
    }
    Ok(lock
        .package
        .into_iter()
        .map(|pkg| package(pkg, &lock.metadata))
        .collect())
}

fn package(pkg: CargoPackage, metadata: &toml::Table) -> ResolvedPackage {
    let checksum = pkg.checksum.as_deref().or_else(|| {
        let source = pkg.source.as_deref()?;
        let key = format!("checksum {} {} ({source})", pkg.name, pkg.version);
        metadata
            .get(&key)?
            .as_str()
            .filter(|value| *value != "<none>")
    });
    let (kind, fingerprint) = resolution(pkg.source.as_deref(), checksum.map(str::to_string));
    ResolvedPackage {
        name: pkg.name,
        version: pkg.version,
        fingerprint,
        kind,
    }
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
            checksum.unwrap_or_else(|| source.to_string()),
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

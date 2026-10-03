use anyhow::{bail, Result};
use std::path::{Component, Path, PathBuf};

pub(crate) fn normalize_catalog_path(raw_path: &str) -> Result<PathBuf> {
    let path = Path::new(raw_path);
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(value) => normalized.push(value),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                bail!("schemaCatalogPath must be a non-empty repository-relative path");
            }
        }
    }
    if normalized.as_os_str().is_empty() {
        bail!("schemaCatalogPath must be a non-empty repository-relative path");
    }
    Ok(normalized)
}

pub(super) fn catalog_path(root: &Path, raw_path: &str) -> Result<PathBuf> {
    Ok(root.join(normalize_catalog_path(raw_path)?))
}

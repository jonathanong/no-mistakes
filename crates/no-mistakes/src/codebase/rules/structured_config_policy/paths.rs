use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub(super) struct CanonicalInventory {
    root: Option<PathBuf>,
    paths: BTreeMap<PathBuf, PathBuf>,
}

impl CanonicalInventory {
    pub(super) fn new(root: &Path, inventory: &[PathBuf]) -> Self {
        let root = root.canonicalize().ok();
        let paths = root
            .as_deref()
            .map(|root| {
                inventory
                    .iter()
                    .filter_map(|path| {
                        canonical_path_in_canonical_root(root, path)
                            .map(|resolved| (path.clone(), resolved))
                    })
                    .collect()
            })
            .unwrap_or_default();
        Self { root, paths }
    }

    pub(super) fn root(&self) -> Option<&Path> {
        self.root.as_deref()
    }

    pub(super) fn path(&self, path: &Path) -> Option<&Path> {
        self.paths.get(path).map(PathBuf::as_path)
    }

    pub(super) fn paths(&self) -> impl Iterator<Item = &Path> {
        self.paths.values().map(PathBuf::as_path)
    }
}

pub(super) fn canonical_path_in_canonical_root(root: &Path, path: &Path) -> Option<PathBuf> {
    let resolved_path = path.canonicalize().ok()?;
    resolved_path.strip_prefix(root).ok()?;
    Some(resolved_path)
}

pub(super) fn contained_in_root(root: &Path, path: &Path) -> bool {
    if path.strip_prefix(root).is_err() {
        return false;
    }
    match (path.canonicalize(), root.canonicalize()) {
        (Ok(resolved), Ok(resolved_root)) => resolved.strip_prefix(resolved_root).is_ok(),
        _ => true,
    }
}

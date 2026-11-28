use super::*;
use std::borrow::Borrow;
use std::hash::{Hash, Hasher};

/// Retain an inventory position instead of cloning its lexical path text.
/// Explicit paths outside prepared inventories still keep their exact spelling.
pub(super) enum CanonicalPath {
    Inventory(Arc<Vec<PathBuf>>, usize),
    Owned(PathBuf),
}

impl CanonicalPath {
    pub(super) fn new(path: &Path, inventories: &[Arc<Vec<PathBuf>>]) -> Self {
        for paths in inventories {
            if let Ok(index) = paths.binary_search_by(|candidate| {
                crate::codebase::ts_source::cmp_os_str_paths(candidate, path)
            }) {
                return Self::Inventory(Arc::clone(paths), index);
            }
        }
        Self::Owned(path.to_path_buf())
    }

    fn path(&self) -> &Path {
        match self {
            Self::Inventory(paths, index) => &paths[*index],
            Self::Owned(path) => path,
        }
    }
}

impl Borrow<Path> for CanonicalPath {
    fn borrow(&self) -> &Path {
        self.path()
    }
}
impl PartialEq for CanonicalPath {
    fn eq(&self, other: &Self) -> bool {
        self.path() == other.path()
    }
}
impl Eq for CanonicalPath {}
impl Hash for CanonicalPath {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.path().hash(state);
    }
}

pub(super) fn inventories(session: &AnalysisSession) -> Vec<Arc<Vec<PathBuf>>> {
    session
        .datasets
        .iter()
        .filter_map(|entry| {
            entry
                .value()
                .get()
                .map(|dataset| dataset.sources_for(entry.key()).inventory().paths())
        })
        .collect()
}

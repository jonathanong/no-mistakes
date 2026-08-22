use rustc_hash::FxHasher;
use std::ops::Deref;

fn interned_path_eq(left: &Arc<Path>, right: &Arc<Path>) -> bool {
    Arc::ptr_eq(left, right) || left.as_os_str() == right.as_os_str()
}

/// Compact interned file identity used as `NodeId::File`.
///
/// `id` is a deterministic Fx hash of the path bytes so standalone
/// [`NodeId::file`] and session-interned [`NodeId::file_in`] stay Hash/Eq
/// compatible. HashMap probes hash this integer instead of the OsStr.
#[derive(Clone, Debug)]
pub struct FileNode {
    id: u64,
    path: Arc<Path>,
}

impl FileNode {
    pub(crate) fn new(path: Arc<Path>) -> Self {
        Self {
            id: file_node_id(path.as_ref()),
            path,
        }
    }

    pub(crate) fn as_arc(&self) -> &Arc<Path> {
        &self.path
    }

    pub(crate) fn clone_arc(&self) -> Arc<Path> {
        Arc::clone(&self.path)
    }
}

fn file_node_id(path: &Path) -> u64 {
    let mut hasher = FxHasher::default();
    path.as_os_str().hash(&mut hasher);
    hasher.finish()
}

impl PartialEq for FileNode {
    fn eq(&self, other: &Self) -> bool {
        interned_path_eq(&self.path, &other.path)
    }
}

impl Eq for FileNode {}

impl Hash for FileNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialOrd for FileNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FileNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.path.as_os_str().cmp(other.path.as_os_str())
    }
}

impl Deref for FileNode {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        &self.path
    }
}

impl AsRef<Path> for FileNode {
    fn as_ref(&self) -> &Path {
        &self.path
    }
}

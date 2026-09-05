use rustc_hash::FxHasher;
use std::ops::Deref;

fn interned_path_eq(left: &Arc<Path>, right: &Arc<Path>) -> bool {
    Arc::ptr_eq(left, right) || left.as_os_str() == right.as_os_str()
}

fn interned_str_eq(left: &Arc<str>, right: &Arc<str>) -> bool {
    Arc::ptr_eq(left, right) || left.as_ref() == right.as_ref()
}

fn fx_content_id<T: Hash + ?Sized>(value: &T) -> u64 {
    let mut hasher = FxHasher::default();
    value.hash(&mut hasher);
    hasher.finish()
}

/// Compact interned file identity used as `NodeId::File`.
///
/// `id` is a deterministic Fx hash of the path bytes so standalone
/// [`NodeId::file`] and session-interned [`NodeId::file_in`] stay Hash/Eq
/// compatible. HashMap probes hash this integer instead of the OsStr.
/// Equality fast-rejects on `id` mismatch; matching ids still compare path
/// bytes so a hash collision cannot collapse distinct paths.
#[derive(Clone, Debug)]
pub struct FileNode {
    id: u64,
    path: Arc<Path>,
}

impl FileNode {
    pub(crate) fn new(path: Arc<Path>) -> Self {
        Self {
            id: fx_content_id(path.as_os_str()),
            path,
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn as_arc(&self) -> &Arc<Path> {
        &self.path
    }

    pub(crate) fn clone_arc(&self) -> Arc<Path> {
        Arc::clone(&self.path)
    }
}

impl PartialEq for FileNode {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && interned_path_eq(&self.path, &other.path)
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

/// Compact interned string identity used as `NodeId` symbol/job/module names.
///
/// `id` is a deterministic Fx hash of the UTF-8 bytes so standalone
/// constructors and session-interned `*_in` constructors stay Hash/Eq
/// compatible. Equal content with distinct `Arc` allocations hashes the same;
/// HashMap probes hash this integer instead of the str bytes.
#[derive(Clone, Debug)]
pub struct InternedStr {
    id: u64,
    value: Arc<str>,
}

impl InternedStr {
    pub(crate) fn new(value: impl Into<Arc<str>>) -> Self {
        let value = value.into();
        Self {
            id: fx_content_id(value.as_ref()),
            value,
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn as_arc(&self) -> &Arc<str> {
        &self.value
    }

    pub(crate) fn clone_arc(&self) -> Arc<str> {
        Arc::clone(&self.value)
    }
}

impl PartialEq for InternedStr {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && interned_str_eq(&self.value, &other.value)
    }
}

impl Eq for InternedStr {}

impl Hash for InternedStr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialOrd for InternedStr {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for InternedStr {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value.as_ref().cmp(other.value.as_ref())
    }
}

impl Deref for InternedStr {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl AsRef<str> for InternedStr {
    fn as_ref(&self) -> &str {
        &self.value
    }
}

impl std::fmt::Display for InternedStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&*self.value, f)
    }
}

impl From<&str> for InternedStr {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for InternedStr {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<Arc<str>> for InternedStr {
    fn from(value: Arc<str>) -> Self {
        Self::new(value)
    }
}

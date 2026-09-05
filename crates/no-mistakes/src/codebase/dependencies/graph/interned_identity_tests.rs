use super::{FileNode, InternedStr};
use std::path::Path;
use std::sync::Arc;

impl FileNode {
    pub(crate) fn from_parts(id: u64, path: Arc<Path>) -> Self {
        Self { id, path }
    }
}

impl InternedStr {
    pub(crate) fn from_parts(id: u64, value: Arc<str>) -> Self {
        Self { id, value }
    }
}

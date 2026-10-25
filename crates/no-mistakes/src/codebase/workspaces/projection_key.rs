/// Retain the immutable catalog owner so invocation-local addresses cannot be
/// recycled while supplemental resolver projections still refer to them.
#[derive(Clone)]
pub(crate) struct WorkspaceProjectionKey(std::sync::Arc<WorkspaceMap>);

impl PartialEq for WorkspaceProjectionKey {
    fn eq(&self, other: &Self) -> bool {
        std::sync::Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for WorkspaceProjectionKey {}
impl std::hash::Hash for WorkspaceProjectionKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::hash::Hash::hash(&(std::sync::Arc::as_ptr(&self.0) as usize), state);
    }
}
impl IndexedWorkspaceMap {
    pub(crate) fn projection_key(&self) -> WorkspaceProjectionKey {
        WorkspaceProjectionKey(std::sync::Arc::clone(&self.workspace))
    }
}

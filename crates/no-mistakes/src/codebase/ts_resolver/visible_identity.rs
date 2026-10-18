/// A retained, invocation-local identity for an exact frozen visible set.
/// Equality never trusts a content fingerprint; the session interns full sets.
#[derive(Clone, Debug)]
pub(crate) struct VisibleScopeIdentity(Arc<()>);

impl VisibleScopeIdentity {
    pub(crate) fn new() -> Self {
        Self(Arc::new(()))
    }
}

impl PartialEq for VisibleScopeIdentity {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for VisibleScopeIdentity {}
impl std::hash::Hash for VisibleScopeIdentity {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::ptr::hash(Arc::as_ptr(&self.0), state);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum ResolverVisibleKey {
    Exact(Vec<PathBuf>),
    Session(VisibleScopeIdentity),
}

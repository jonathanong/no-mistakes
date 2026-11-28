#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportClassification {
    resolver_target: Option<PathBuf>,
    workspace_target: Option<PathBuf>,
    workspace_recognized: bool,
}

impl ImportClassification {
    pub fn preferred_path(&self) -> Option<&Path> {
        self.resolver_target
            .as_deref()
            .or(self.workspace_target.as_deref())
    }

    pub fn resolver_path(&self) -> Option<&Path> {
        self.resolver_target.as_deref()
    }

    pub fn workspace_path(&self) -> Option<&Path> {
        self.workspace_target.as_deref()
    }

    pub fn is_unresolved_external(&self) -> bool {
        self.resolver_target.is_none()
            && self.workspace_target.is_none()
            && !self.workspace_recognized
    }

    #[cfg(any(test, feature = "test-instrumentation"))]
    #[doc(hidden)]
    pub fn unresolved() -> Self {
        Self {
            resolver_target: None,
            workspace_target: None,
            workspace_recognized: false,
        }
    }
}

/// Request-scoped memo table shared by graph consumers that classify imports.
///
/// The cache owns only resolution outcomes, so sharing it does not make lazy
/// traversal prepare any other graph consumer.
#[derive(Default)]
pub struct ImportResolutionCache {
    raw_entries: DashMap<ResolveKey, Option<PathBuf>, rustc_hash::FxBuildHasher>,
    final_entries: DashMap<ClassificationKey, ImportClassification, rustc_hash::FxBuildHasher>,
    classifications: std::sync::atomic::AtomicUsize,
    requests: std::sync::atomic::AtomicUsize,
}

impl ImportResolutionCache {
    pub fn clear(&self) {
        self.raw_entries.clear();
        self.final_entries.clear();
    }

    #[cfg(any(test, feature = "test-instrumentation"))]
    #[doc(hidden)]
    pub fn classification_count(&self) -> usize {
        self.classifications
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    #[cfg(any(test, feature = "test-instrumentation"))]
    #[doc(hidden)]
    pub fn request_count(&self) -> usize {
        self.requests.load(std::sync::atomic::Ordering::Relaxed)
    }
}

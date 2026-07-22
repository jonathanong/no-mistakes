use super::PlaywrightModuleResolution;
use std::sync::atomic::Ordering;
use std::sync::Arc;

impl PlaywrightModuleResolution {
    pub(crate) fn catalog_instrumentation(&self) -> Option<(bool, usize, usize, usize)> {
        self.catalog_resolver.as_ref().map(|catalog| {
            (
                Arc::ptr_eq(&self.remapper, &catalog.universe),
                catalog.scope_selections.load(Ordering::Relaxed),
                catalog.scope_builds.load(Ordering::Relaxed),
                catalog.classifications.len(),
            )
        })
    }
}

use super::AnalysisSession;
pub use no_mistakes_ts_analysis::path_interner::PathInterner;
use std::sync::Arc;

impl AnalysisSession {
    /// Borrow the request-owned intern table.
    pub fn interner(&self) -> &PathInterner {
        &self.interner
    }

    /// Clone the request-owned intern table handle for graph workers.
    pub(crate) fn interner_arc(&self) -> Arc<PathInterner> {
        Arc::clone(&self.interner)
    }
}

#[cfg(test)]
mod tests;

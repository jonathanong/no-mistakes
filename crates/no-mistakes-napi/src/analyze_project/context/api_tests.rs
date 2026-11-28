use super::*;

impl AnalyzeProjectContext {
    pub(crate) fn initialized_playwright_analysis_count(&self) -> usize {
        self.scopes
            .values()
            .map(|scope| {
                scope
                    .playwright_analyses
                    .lock()
                    .expect("report cache is poisoned")
                    .values()
                    .filter(|cell| cell.get().is_some())
                    .count()
            })
            .sum()
    }

    pub(crate) fn graph_build_count(&self) -> usize {
        self.scopes
            .values()
            .map(|scope| scope.traversal.graph_build_count())
            .sum()
    }
}

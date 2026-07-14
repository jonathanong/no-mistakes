use super::*;

impl AnalyzeProjectContext {
    pub(crate) fn graph_build_count(&self) -> usize {
        self.scopes
            .values()
            .map(|scope| scope.traversal.graph_builds)
            .sum()
    }
}

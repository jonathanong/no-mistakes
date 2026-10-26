use super::CheckFactPlan;
impl CheckFactPlan {
    pub(crate) fn collect_lint_suppressions(
        &self,
        path: &std::path::Path,
        program: &oxc_ast::ast::Program<'_>,
        source: &str,
    ) -> Option<Vec<crate::codebase::rules::forbidden_lint_suppressions::Suppression>> {
        self.lint_suppression_paths
            .contains(path)
            .then(|| crate::codebase::rules::forbidden_lint_suppressions::collect(program, source))
    }
}

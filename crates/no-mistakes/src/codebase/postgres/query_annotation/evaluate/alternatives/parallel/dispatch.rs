use super::*;
impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn alternatives(
        &mut self,
        arms: &[Expr],
        path: &Path,
        env: &Environment,
        depth: u8,
        generic: bool,
    ) -> Value {
        if let Some(handler) = self.parallel_arms {
            if let Some(value) = handler(self, arms, path, env, depth, generic) {
                return value;
            }
        }
        self.alternatives_prepared(arms, path, env, depth, generic, None)
    }
}

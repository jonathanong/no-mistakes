use super::ScopeVisitor;

impl ScopeVisitor<'_> {
    /// True when `name` is an outer function's binding mutated from a nested
    /// function. Walking a nested declaration is not executing it, so those
    /// appends must not rewrite the outer SQL.
    pub(in crate::codebase::postgres::embedded::walk) fn append_crosses_function(
        &self,
        name: &str,
    ) -> bool {
        let Some(&fn_start) = self.function_scopes.last() else {
            return false;
        };
        self.scopes
            .iter()
            .enumerate()
            .rev()
            .find_map(|(index, scope)| scope.contains_key(name).then_some(index))
            .is_some_and(|index| index < fn_start)
    }
}

use super::*;
use oxc_ast::ast::{Expression, ReturnStatement};

impl ImportCollector {
    pub(in super::super::super::super) fn allow_metadata_row_return(
        &mut self,
        statement: &ReturnStatement<'_>,
    ) {
        let Some(expression) = &statement.argument else {
            return;
        };
        let Expression::Identifier(identifier) =
            crate::codebase::ts_source::unwrap_ts_wrappers(expression)
        else {
            return;
        };
        let Some(scope) = self.callee_binding_scope(identifier.name.as_str()) else {
            return;
        };
        if !self
            .metadata_rows
            .lookups
            .get(&scope)
            .is_some_and(|names| names.contains(identifier.name.as_str()))
        {
            return;
        }
        let found = self.value_flow.bindings.iter().any(|binding| {
            binding.name == identifier.name.as_str()
                && binding.scope == scope
                && binding.owner == self.current_function_id()
                && binding.declared_at < identifier.span.start
                && binding
                    .invalidated_at
                    .is_none_or(|end| identifier.span.start < end)
        });
        if found {
            self.closed_registry_reads.insert(identifier.span.start);
            self.metadata_rows.allowed.insert(identifier.span.start);
        }
    }
}

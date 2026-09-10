use super::{resolve, BindingState, EmbeddedSqlKind, ScopeVisitor};
use oxc_ast::ast::{BindingPattern, FormalParameters};
use std::collections::HashMap;

impl ScopeVisitor<'_> {
    pub(super) fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub(super) fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    pub(super) fn current_scope(&mut self) -> Option<&mut HashMap<String, BindingState>> {
        self.scopes.last_mut()
    }

    pub(super) fn lookup(&self, name: &str) -> Option<BindingState> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).cloned())
    }

    /// Whether `name` is bound at a scope more deeply nested than the
    /// top-level program scope — a real lexical shadow of a same-named
    /// top-level helper. A match found only in the outermost scope is the
    /// helper's own top-level declaration (JS/TS forbids redeclaring a name
    /// twice in one scope), not a shadow, and must not block resolving it.
    pub(super) fn shadowed_locally(&self, name: &str) -> bool {
        self.scopes
            .get(1..)
            .is_some_and(|nested| nested.iter().any(|scope| scope.contains_key(name)))
    }

    pub(super) fn bind_param(&mut self, pattern: &BindingPattern<'_>) {
        let mut names = Vec::new();
        resolve::for_each_bound_name(pattern, &mut |name| names.push(name.to_string()));
        for name in names {
            if let Some(scope) = self.current_scope() {
                scope.insert(
                    name,
                    BindingState {
                        sql: None,
                        kind: EmbeddedSqlKind::Dynamic,
                        line: 0,
                    },
                );
            }
        }
    }

    /// Binds a named function expression's own self-reference into its
    /// just-pushed body scope, as a shadow marker only — this is never a
    /// same-file helper `LocalFunctions` resolves calls through by this
    /// name (it already requires the const/expression names to match, and
    /// otherwise drops the binding), so recording it here exists solely to
    /// make `shadowed_locally` see the rebind.
    pub(super) fn bind_self_name(&mut self, name: &str) {
        if let Some(scope) = self.current_scope() {
            scope.insert(
                name.to_string(),
                BindingState {
                    sql: None,
                    kind: EmbeddedSqlKind::Dynamic,
                    line: 0,
                },
            );
        }
    }

    pub(super) fn mark_dynamic(&mut self, name: &str) {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(binding) = scope.get_mut(name) {
                binding.kind = EmbeddedSqlKind::Dynamic;
                binding.sql = None;
                return;
            }
        }
    }

    /// Conditional appends must not compose as if the branch always ran, but
    /// wiping recovered SQL would make non-INSERT executors opaque. Keep the
    /// text and mark Dynamic: INSERT fails closed, SELECT/UPDATE stay ignored.
    pub(super) fn mark_dynamic_keep_sql(&mut self, name: &str) {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(binding) = scope.get_mut(name) {
                binding.kind = EmbeddedSqlKind::Dynamic;
                return;
            }
        }
    }

    pub(super) fn record_params(&mut self, params: &FormalParameters<'_>) {
        for param in &params.items {
            self.bind_param(&param.pattern);
        }
        if let Some(rest) = &params.rest {
            self.bind_param(&rest.rest.argument);
        }
    }

    pub(super) fn with_control_flow(&mut self, walk: impl FnOnce(&mut Self)) {
        self.control_depth += 1;
        walk(self);
        self.control_depth = self.control_depth.saturating_sub(1);
    }

    pub(super) fn enter_function(&mut self) {
        self.function_scopes
            .push(self.scopes.len().saturating_sub(1));
    }

    pub(super) fn leave_function(&mut self) {
        self.function_scopes.pop();
    }

    pub(super) fn with_loop(&mut self, walk: impl FnOnce(&mut Self)) {
        self.loop_depth += 1;
        walk(self);
        self.loop_depth = self.loop_depth.saturating_sub(1);
    }

    /// True when `name` is an outer function's binding mutated from a nested
    /// function. Walking a nested declaration is not executing it, so those
    /// appends must not rewrite the outer SQL.
    pub(super) fn append_crosses_function(&self, name: &str) -> bool {
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

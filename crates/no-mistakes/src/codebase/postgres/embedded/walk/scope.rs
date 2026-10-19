use super::super::EmbeddedSqlKind;
use super::{resolve, BindingState, ScopeVisitor};
use oxc_ast::ast::{BindingPattern, FormalParameters};
use oxc_span::GetSpan;
use std::collections::HashMap;

impl ScopeVisitor<'_> {
    pub(super) fn push_scope(&mut self) {
        self.literal_objects.push_frame();
        self.member_query_proof.next_scope_id += 1;
        self.member_query_proof
            .scope_ids
            .push(self.member_query_proof.next_scope_id);
        self.scopes.push(HashMap::new());
        self.relative_proofs.push(HashMap::new());
        self.namespace_proofs.push(Default::default());
    }

    pub(super) fn pop_scope(&mut self) {
        self.literal_objects.pop_frame();
        self.member_query_proof.scope_ids.pop();
        self.scopes.pop();
        self.relative_proofs.pop();
        self.namespace_proofs.pop();
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

    pub(super) fn is_sql_builder(&self, name: &str) -> bool {
        self.lookup(name).is_some_and(|binding| binding.sql_builder)
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

    pub(super) fn bind_param(&mut self, pattern: &BindingPattern<'_>, sql_builder: bool) {
        let mut names = Vec::new();
        resolve::for_each_bound_name(pattern, &mut |name| names.push(name.to_string()));
        for name in names {
            let executor = self
                .scoped
                .contains_declaration(&name, pattern.span().start);
            let relative_executor_proof =
                self.relative_declaration_proof(&name, pattern.span().start);
            if let Some(scope) = self.current_scope() {
                scope.insert(
                    name.clone(),
                    BindingState {
                        executor_alias: executor
                            .then_some(true)
                            .or(relative_executor_proof.map(|_| false)),
                        builder_identity: None,
                        builder_effect_revision: 0,
                        condition_key: None,
                        variants: None,
                        sql: None,
                        kind: EmbeddedSqlKind::Dynamic,
                        line: 0,
                        initialized: false,
                        sql_builder,
                        sql_source_positions: Vec::new(),
                    },
                );
            }
            if let Some(index) = relative_executor_proof {
                self.relative_proofs
                    .last_mut()
                    .expect("scope")
                    .insert(name, index);
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
                    executor_alias: None,
                    builder_identity: None,
                    builder_effect_revision: 0,
                    condition_key: None,
                    variants: None,
                    sql: None,
                    kind: EmbeddedSqlKind::Dynamic,
                    line: 0,
                    initialized: false,
                    sql_builder: false,
                    sql_source_positions: Vec::new(),
                },
            );
        }
    }

    pub(super) fn mark_dynamic(&mut self, name: &str) {
        self.forget_receiver_alias(name);
        for scope in self.scopes.iter_mut().rev() {
            if let Some(binding) = scope.get_mut(name) {
                binding.executor_alias = None;
                binding.initialized = true;
                binding.kind = EmbeddedSqlKind::Dynamic;
                binding.condition_key = None;
                binding.variants = None;
                binding.builder_identity = None;
                binding.sql_builder = false;
                binding.sql = None;
                binding.sql_source_positions.clear();
                return;
            }
        }
    }

    /// Conditional appends must not compose as if the branch always ran, but
    /// wiping recovered SQL would make non-INSERT executors opaque. Keep the
    /// text and mark Dynamic: INSERT fails closed, SELECT/UPDATE stay ignored.
    pub(super) fn mark_dynamic_keep_known_statement(&mut self, name: &str) {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(binding) = scope.get_mut(name) {
                mark_binding_dynamic_keep_known_statement(binding);
                return;
            }
        }
    }

    pub(super) fn record_params(&mut self, params: &FormalParameters<'_>) {
        for param in &params.items {
            self.bind_param(
                &param.pattern,
                params::param_is_sql_statement(param, &self.sql_statement_types),
            );
        }
        if let Some(rest) = &params.rest {
            self.bind_param(&rest.rest.argument, false);
        }
    }

    pub(super) fn with_control_flow(&mut self, walk: impl FnOnce(&mut Self)) {
        self.control_depth += 1;
        walk(self);
        self.control_depth = self.control_depth.saturating_sub(1);
    }

    pub(super) fn enter_function(&mut self) {
        self.member_query_proof.next_function_id += 1;
        self.member_query_proof
            .function_ids
            .push(self.member_query_proof.next_function_id);
        self.function_scopes
            .push(self.scopes.len().saturating_sub(1));
    }

    pub(super) fn leave_function(&mut self) {
        self.member_query_proof.function_ids.pop();
        self.function_scopes.pop();
    }

    pub(super) fn with_loop(&mut self, walk: impl FnOnce(&mut Self)) {
        self.loop_depth += 1;
        walk(self);
        self.loop_depth = self.loop_depth.saturating_sub(1);
    }
}

mod function;
mod known_statement;
mod params;
pub(super) use known_statement::mark_binding_dynamic_keep_known_statement;
pub(crate) use known_statement::{has_diagnostic_leading_statement, has_known_leading_statement};

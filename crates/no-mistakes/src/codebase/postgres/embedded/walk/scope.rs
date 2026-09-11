use super::{resolve, BindingState, EmbeddedSqlKind, ScopeVisitor};
use oxc_ast::ast::{BindingPattern, FormalParameters, TSType, TSTypeName};
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
            if let Some(scope) = self.current_scope() {
                scope.insert(
                    name,
                    BindingState {
                        sql: None,
                        kind: EmbeddedSqlKind::Dynamic,
                        line: 0,
                        sql_builder,
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
                    sql_builder: false,
                },
            );
        }
    }

    pub(super) fn mark_dynamic(&mut self, name: &str) {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(binding) = scope.get_mut(name) {
                binding.kind = EmbeddedSqlKind::Dynamic;
                binding.sql_builder = false;
                binding.sql = None;
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
                param_is_sql_statement(param, &self.sql_statement_types),
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

fn param_is_sql_statement(
    param: &oxc_ast::ast::FormalParameter<'_>,
    bindings: &std::collections::HashSet<String>,
) -> bool {
    let Some(annotation) = &param.type_annotation else {
        return false;
    };
    let TSType::TSTypeReference(reference) = &annotation.type_annotation else {
        return false;
    };
    matches!(
        &reference.type_name,
        TSTypeName::IdentifierReference(identifier) if bindings.contains(identifier.name.as_str())
    )
}

pub(super) fn mark_binding_dynamic_keep_known_statement(binding: &mut BindingState) {
    binding.kind = EmbeddedSqlKind::Dynamic;
    if binding
        .sql
        .as_deref()
        .is_none_or(|sql| !has_known_leading_statement(sql))
    {
        binding.sql = None;
    }
}

pub(super) fn has_known_leading_statement(mut sql: &str) -> bool {
    loop {
        sql = sql.trim_start();
        if let Some(comment) = sql.strip_prefix("--") {
            let Some((_, rest)) = comment.split_once('\n') else {
                return false;
            };
            sql = rest;
            continue;
        }
        if let Some(comment) = sql.strip_prefix("/*") {
            let Some((_, rest)) = comment.split_once("*/") else {
                return false;
            };
            sql = rest;
            continue;
        }
        break;
    }

    let keyword = sql
        .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        .next()
        .unwrap_or_default();
    ["select", "update", "insert", "delete", "merge"]
        .iter()
        .any(|candidate| keyword.eq_ignore_ascii_case(candidate))
}

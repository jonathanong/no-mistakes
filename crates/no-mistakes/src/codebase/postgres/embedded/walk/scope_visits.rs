use super::{resolve, ScopeVisitor};
use oxc_ast::ast;
use oxc_ast_visit::walk;
use oxc_syntax::scope::ScopeFlags;

impl<'a> ScopeVisitor<'a> {
    pub(super) fn walk_block_scope(&mut self, block: &ast::BlockStatement<'a>) {
        self.push_scope();
        resolve::record_statements(&block.body, self);
        walk::walk_block_statement(self, block);
        self.pop_scope();
    }

    pub(super) fn walk_namespace_scope(&mut self, block: &ast::TSModuleBlock<'a>) {
        self.push_scope();
        self.shadow_namespace_types(block);
        self.var_scopes.push(self.scopes.len().saturating_sub(1));
        resolve::hoist_vars(&block.body, self);
        resolve::record_statements(&block.body, self);
        walk::walk_ts_module_block(self, block);
        self.var_scopes.pop();
        self.pop_scope();
    }

    pub(super) fn walk_catch_clause_scope(&mut self, clause: &oxc_ast::ast::CatchClause<'a>) {
        self.push_scope();
        if let Some(param) = &clause.param {
            self.bind_param(&param.pattern, false);
        }
        walk::walk_catch_clause(self, clause);
        self.pop_scope();
    }

    pub(super) fn walk_function_scope(&mut self, function: &ast::Function<'a>, flags: ScopeFlags) {
        // Expression names are private self-references, not externally callable keys.
        let declaration_name = (function.r#type != ast::FunctionType::FunctionExpression)
            .then_some(function.id.as_ref())
            .flatten()
            .map(|id| id.name.as_str());
        let mut writes = super::stable_binds::Writes::default();
        super::stable_binds::parameter_effects(&mut writes, &function.params);
        if let Some(body) = &function.body {
            oxc_ast_visit::Visit::visit_function_body(&mut writes, body);
        }
        self.stable_primitives
            .extend(writes.parameters(&function.params, |ty| self.is_closed_relation_type(ty)));
        self.stable_parameter_fields
            .extend(writes.fields(&function.params));
        self.enter_plain_function(declaration_name, function.span.start);
        self.push_scope();
        self.shadow_parameter_generics(function.type_parameters.as_deref());
        self.record_params(&function.params);
        // A named function expression shadows outer helpers only in its own
        // body. LocalFunctions does not resolve through this self-reference,
        // so record a shadow marker in the scope just pushed.
        if function.r#type == ast::FunctionType::FunctionExpression {
            if let Some(id) = &function.id {
                self.bind_self_name(id.name.as_str());
                self.alias_plain_self(id.name.as_str());
            }
        }
        self.enter_function();
        let set_shadow = function
            .body
            .as_ref()
            .is_some_and(|body| super::guarded_sql::shadows_set(&function.params, body));
        let guarded = (!self.head_calls_only
            && self.set_intrinsic_safe
            && self.lookup("Set").is_none()
            && !set_shadow)
            .then(|| {
                super::guarded_sql::GuardedCase::from_function(function, self.function_scopes.len())
            })
            .flatten();
        let inserted = guarded.is_some();
        if let Some(guarded) = guarded {
            self.guarded_cases.push(guarded);
        }
        walk::walk_function(self, function, flags);
        if inserted {
            self.guarded_cases.pop();
        }
        self.leave_plain_function();
        self.leave_function();
        self.pop_scope();
    }

    pub(super) fn walk_class_scope(&mut self, class: &oxc_ast::ast::Class<'a>) {
        let private_name = (class.r#type == oxc_ast::ast::ClassType::ClassExpression)
            .then_some(class.id.as_ref())
            .flatten();
        let scoped = private_name.is_some() || class.type_parameters.is_some();
        if scoped {
            self.push_scope();
            self.shadow_parameter_generics(class.type_parameters.as_deref());
        }
        if let Some(name) = private_name {
            self.bind_self_name(name.name.as_str());
            self.shadow_parameter_type(name.name.as_str());
        }
        // The private class name shadows outer bindings in heritage expressions,
        // computed keys and method bodies; it must not escape the expression.
        walk::walk_class(self, class);
        if scoped {
            self.pop_scope();
        }
    }

    pub(super) fn walk_static_block_scope(&mut self, block: &oxc_ast::ast::StaticBlock<'a>) {
        // Class static blocks are also independent var ownership boundaries.
        self.push_scope();
        self.enter_function();
        self.enter_plain_function(None, block.span.start);
        resolve::hoist_vars(&block.body, self);
        resolve::record_statements(&block.body, self);
        walk::walk_static_block(self, block);
        self.leave_plain_function();
        self.leave_function();
        self.pop_scope();
    }

    pub(super) fn walk_arrow_scope(&mut self, arrow: &ast::ArrowFunctionExpression<'a>) {
        let mut writes = super::stable_binds::Writes::default();
        super::stable_binds::parameter_effects(&mut writes, &arrow.params);
        oxc_ast_visit::Visit::visit_arrow_function_body(&mut writes, &arrow.body);
        self.stable_primitives
            .extend(writes.parameters(&arrow.params, |ty| self.is_closed_relation_type(ty)));
        self.stable_parameter_fields
            .extend(writes.fields(&arrow.params));
        self.push_scope();
        self.shadow_parameter_generics(arrow.type_parameters.as_deref());
        self.record_params(&arrow.params);
        self.enter_function();
        self.enter_plain_function(None, arrow.span.start);
        if let Some(result) = arrow.body.as_expression() {
            // A concise return can expose a receiver to later property writes.
            self.revoke_query_receiver(result);
        }
        walk::walk_arrow_function_expression(self, arrow);
        if let Some(result) = arrow.body.as_expression() {
            self.note_returned_builder_fragments(result);
        }
        self.leave_plain_function();
        self.leave_function();
        self.pop_scope();
    }
}

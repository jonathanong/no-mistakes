use super::*;
impl<'a> Visit<'a> for Typed {
    fn visit_if_statement(&mut self, value: &oxc_ast::ast::IfStatement<'a>) {
        walk::walk_if_statement(self, value);
        self.freeze.test(&value.test);
    }
    fn visit_while_statement(&mut self, value: &oxc_ast::ast::WhileStatement<'a>) {
        walk::walk_while_statement(self, value);
        self.freeze.test(&value.test);
    }
    fn visit_do_while_statement(&mut self, value: &oxc_ast::ast::DoWhileStatement<'a>) {
        walk::walk_do_while_statement(self, value);
        self.freeze.test(&value.test);
    }
    fn visit_for_statement(&mut self, value: &oxc_ast::ast::ForStatement<'a>) {
        walk::walk_for_statement(self, value);
        if let Some(test) = &value.test {
            self.freeze.test(test);
        }
    }
    fn visit_expression(&mut self, value: &Expression<'a>) {
        walk::walk_expression(self, value);
        self.freeze.expression(value, &self.bindings);
    }
    fn visit_function(
        &mut self,
        value: &oxc_ast::ast::Function<'a>,
        flags: oxc_syntax::scope::ScopeFlags,
    ) {
        let id = value
            .id
            .as_ref()
            .and_then(|id| self.bindings.get(&span(id.span)).copied());
        let owner = self.freeze.enter(span(value.span), id, false);
        walk::walk_function(self, value, flags);
        self.freeze.restore(owner);
    }
    fn visit_arrow_function_expression(
        &mut self,
        value: &oxc_ast::ast::ArrowFunctionExpression<'a>,
    ) {
        let owner = self.freeze.enter(span(value.span), None, true);
        walk::walk_arrow_function_expression(self, value);
        if let Some(expression) = value.body.as_expression() {
            self.freeze.returned(expression);
        }
        self.freeze.restore(owner);
    }
    fn visit_return_statement(&mut self, value: &oxc_ast::ast::ReturnStatement<'a>) {
        walk::walk_return_statement(self, value);
        if let Some(expression) = &value.argument {
            self.freeze.returned(expression);
        }
    }

    fn visit_variable_declarator(&mut self, variable: &oxc_ast::ast::VariableDeclarator<'a>) {
        if let Some(annotation) = &variable.type_annotation {
            self.unknown_pattern(&variable.id, &annotation.type_annotation);
        }
        self.factory_binding(variable);
        self.freeze.variable(variable, &self.bindings);
        walk::walk_variable_declarator(self, variable);
    }
    fn visit_formal_parameters(&mut self, parameters: &oxc_ast::ast::FormalParameters<'a>) {
        self.parameters(parameters);
        walk::walk_formal_parameters(self, parameters);
    }
    fn visit_variable_declaration(&mut self, value: &VariableDeclaration<'a>) {
        let old = self.immutable;
        self.immutable = matches!(
            value.kind,
            oxc_ast::ast::VariableDeclarationKind::Const
                | oxc_ast::ast::VariableDeclarationKind::Using
                | oxc_ast::ast::VariableDeclarationKind::AwaitUsing
        );
        walk::walk_variable_declaration(self, value);
        self.immutable = old;
    }
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        self.wrapper(call);
        walk::walk_call_expression(self, call);
    }
    fn visit_simple_assignment_target(
        &mut self,
        target: &oxc_ast::ast::SimpleAssignmentTarget<'a>,
    ) {
        self.freeze.write(target, &self.bindings);
        if let Some(at) = super::template_binds::assignment_root(target) {
            self.write(at);
        }
        if let oxc_ast::ast::SimpleAssignmentTarget::ComputedMemberExpression(value) = target {
            if let Some((root, _)) = chain(&value.expression) {
                self.transports.insert(root);
            }
        }
        walk::walk_simple_assignment_target(self, target);
    }
    fn visit_assignment_target_property_identifier(
        &mut self,
        target: &oxc_ast::ast::AssignmentTargetPropertyIdentifier<'a>,
    ) {
        self.write(span(target.binding.span));
        self.freeze.global_write(
            target.binding.name.as_str(),
            span(target.binding.span),
            &self.bindings,
        );
        walk::walk_assignment_target_property_identifier(self, target);
    }
}

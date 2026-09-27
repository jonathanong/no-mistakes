impl<'a> Visit<'a> for Receivers {
    fn visit_binding_identifier(&mut self, identifier: &BindingIdentifier<'a>) {
        self.declaration_scopes
            .insert(identifier.name.to_string(), self.scope.clone());
        *self
            .declarations
            .entry(identifier.name.to_string())
            .or_default() += 1;
    }

    fn visit_variable_declarator(&mut self, declaration: &VariableDeclarator<'a>) {
        if let oxc_ast::ast::BindingPattern::BindingIdentifier(binding) = &declaration.id {
            self.assign(binding.name.as_str(), declaration.init.as_ref());
        }
        walk::walk_variable_declarator(self, declaration);
    }

    fn visit_assignment_expression(&mut self, assignment: &AssignmentExpression<'a>) {
        use oxc_ast::ast::AssignmentTarget;
        match &assignment.left {
            AssignmentTarget::AssignmentTargetIdentifier(binding) => {
                self.assign(binding.name.as_str(), Some(&assignment.right));
            }
            AssignmentTarget::StaticMemberExpression(member) => {
                self.invalidate_object(&member.object)
            }
            AssignmentTarget::ComputedMemberExpression(member) => {
                self.invalidate_object(&member.object)
            }
            _ => MutationTargets(self).visit_assignment_target(&assignment.left),
        }
        walk::walk_assignment_expression(self, assignment);
    }

    fn visit_call_expression(&mut self, call: &oxc_ast::ast::CallExpression<'a>) {
        self.visit_initializing_call(call);
    }
    fn visit_function(
        &mut self,
        function: &oxc_ast::ast::Function<'a>,
        flags: oxc_syntax::scope::ScopeFlags,
    ) {
        self.with_initialization_context(false, false, |this| {
            walk::walk_function(this, function, flags)
        });
    }
    fn visit_arrow_function_expression(
        &mut self,
        function: &oxc_ast::ast::ArrowFunctionExpression<'a>,
    ) {
        self.with_initialization_context(false, false, |this| {
            walk::walk_arrow_function_expression(this, function)
        });
    }
    fn visit_if_statement(&mut self, statement: &oxc_ast::ast::IfStatement<'a>) {
        self.with_initialization_context(false, false, |this| {
            walk::walk_if_statement(this, statement)
        });
    }
    fn visit_block_statement(&mut self, statement: &oxc_ast::ast::BlockStatement<'a>) {
        self.with_initialization_context(false, false, |this| {
            walk::walk_block_statement(this, statement)
        });
    }
    fn visit_for_statement(&mut self, statement: &oxc_ast::ast::ForStatement<'a>) {
        self.with_initialization_context(false, false, |this| {
            walk::walk_for_statement(this, statement)
        });
    }
    fn visit_for_in_statement(&mut self, statement: &oxc_ast::ast::ForInStatement<'a>) {
        self.with_initialization_context(false, false, |this| {
            walk::walk_for_in_statement(this, statement)
        });
    }
    fn visit_for_of_statement(&mut self, statement: &oxc_ast::ast::ForOfStatement<'a>) {
        self.with_initialization_context(false, false, |this| {
            walk::walk_for_of_statement(this, statement)
        });
    }
    fn visit_while_statement(&mut self, statement: &oxc_ast::ast::WhileStatement<'a>) {
        self.with_initialization_context(false, false, |this| {
            walk::walk_while_statement(this, statement)
        });
    }
    fn visit_do_while_statement(&mut self, statement: &oxc_ast::ast::DoWhileStatement<'a>) {
        self.with_initialization_context(false, false, |this| {
            walk::walk_do_while_statement(this, statement)
        });
    }
    fn visit_switch_statement(&mut self, statement: &oxc_ast::ast::SwitchStatement<'a>) {
        self.with_initialization_context(false, false, |this| {
            walk::walk_switch_statement(this, statement)
        });
    }
    fn visit_logical_expression(&mut self, expression: &oxc_ast::ast::LogicalExpression<'a>) {
        self.with_initialization_context(false, false, |this| {
            walk::walk_logical_expression(this, expression)
        });
    }
    fn visit_conditional_expression(
        &mut self,
        expression: &oxc_ast::ast::ConditionalExpression<'a>,
    ) {
        self.with_initialization_context(false, false, |this| {
            walk::walk_conditional_expression(this, expression)
        });
    }
}

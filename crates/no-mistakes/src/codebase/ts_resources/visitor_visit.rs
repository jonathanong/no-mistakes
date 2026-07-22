use super::assignment_targets::assignment_target_names;
use super::ResourceVisitor;
use oxc_ast::ast::{
    ArrowFunctionExpression, AssignmentExpression, BlockStatement, CatchClause, Class,
    ExportDefaultDeclaration, ForInStatement, ForOfStatement, ForStatement, ForStatementInit,
    ForStatementLeft, Function, ImportDeclaration, MethodDefinition, ObjectProperty,
    SwitchStatement, VariableDeclaration, VariableDeclarationKind, VariableDeclarator,
};
use oxc_ast_visit::{walk, Visit};

impl<'a> ResourceVisitor<'a> {
    fn push_loop_lexical_scope(&mut self, declaration: Option<&VariableDeclaration<'_>>) {
        self.push_lexical_scope();
        let Some(declaration) = declaration else {
            return;
        };
        if declaration.kind != VariableDeclarationKind::Var {
            for declarator in &declaration.declarations {
                self.shadow_pattern(&declarator.id);
            }
        }
    }
}

impl<'a> Visit<'a> for ResourceVisitor<'a> {
    fn visit_import_declaration(&mut self, import: &ImportDeclaration<'a>) {
        self.register_import(import);
        walk::walk_import_declaration(self, import);
    }

    fn visit_variable_declarator(&mut self, declarator: &VariableDeclarator<'a>) {
        self.visit_variable_declarator_impl(declarator);
    }

    fn visit_function(&mut self, function: &Function<'a>, flags: oxc_syntax::scope::ScopeFlags) {
        self.visit_scoped_function(
            function.id.as_ref().map(|id| id.name.to_string()),
            function,
            flags,
        );
    }

    fn visit_arrow_function_expression(&mut self, arrow: &ArrowFunctionExpression<'a>) {
        self.visit_scoped_arrow(None, arrow);
    }

    fn visit_method_definition(&mut self, method: &MethodDefinition<'a>) {
        self.visit_method_definition_impl(method);
    }

    fn visit_object_property(&mut self, property: &ObjectProperty<'a>) {
        self.visit_object_property_impl(property);
    }

    fn visit_class(&mut self, class: &Class<'a>) {
        if let Some(id) = class.id.as_ref().filter(|_| self.function_stack.is_empty()) {
            self.visit_default_class(id.name.to_string(), class);
        } else {
            walk::walk_class(self, class);
        }
    }

    fn visit_export_default_declaration(&mut self, export: &ExportDefaultDeclaration<'a>) {
        self.visit_export_default_declaration_impl(export);
    }

    fn visit_block_statement(&mut self, block: &BlockStatement<'a>) {
        self.push_lexical_scope();
        self.predeclare_statement_bindings(&block.body);
        walk::walk_block_statement(self, block);
        self.pop_lexical_scope();
    }

    fn visit_catch_clause(&mut self, clause: &CatchClause<'a>) {
        self.push_lexical_scope();
        if let Some(parameter) = &clause.param {
            self.shadow_pattern(&parameter.pattern);
        }
        walk::walk_catch_clause(self, clause);
        self.pop_lexical_scope();
    }

    fn visit_switch_statement(&mut self, switch: &SwitchStatement<'a>) {
        self.push_lexical_scope();
        for case in &switch.cases {
            self.predeclare_statement_bindings(&case.consequent);
        }
        walk::walk_switch_statement(self, switch);
        self.pop_lexical_scope();
    }

    fn visit_for_statement(&mut self, statement: &ForStatement<'a>) {
        let declaration = match &statement.init {
            Some(ForStatementInit::VariableDeclaration(declaration)) => Some(declaration.as_ref()),
            _ => None,
        };
        self.push_loop_lexical_scope(declaration);
        walk::walk_for_statement(self, statement);
        self.pop_lexical_scope();
    }

    fn visit_for_in_statement(&mut self, statement: &ForInStatement<'a>) {
        let declaration = match &statement.left {
            ForStatementLeft::VariableDeclaration(declaration) => Some(declaration.as_ref()),
            _ => None,
        };
        self.push_loop_lexical_scope(declaration);
        walk::walk_for_in_statement(self, statement);
        self.pop_lexical_scope();
    }

    fn visit_for_of_statement(&mut self, statement: &ForOfStatement<'a>) {
        let declaration = match &statement.left {
            ForStatementLeft::VariableDeclaration(declaration) => Some(declaration.as_ref()),
            _ => None,
        };
        self.push_loop_lexical_scope(declaration);
        walk::walk_for_of_statement(self, statement);
        self.pop_lexical_scope();
    }

    fn visit_call_expression(&mut self, call: &oxc_ast::ast::CallExpression<'a>) {
        self.record_call(call);
        walk::walk_call_expression(self, call);
    }

    fn visit_assignment_expression(&mut self, assignment: &AssignmentExpression<'a>) {
        for name in assignment_target_names(&assignment.left) {
            self.invalidate_binding(&name);
        }
        walk::walk_assignment_expression(self, assignment);
    }
}

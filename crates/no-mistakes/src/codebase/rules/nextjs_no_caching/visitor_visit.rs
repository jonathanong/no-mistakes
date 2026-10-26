use super::visitor::NextjsCachingVisitor;
use oxc_ast::ast::{
    ArrowFunctionExpression, AssignmentExpression, BlockStatement, CallExpression, CatchClause,
    Class, ForInStatement, ForOfStatement, ForStatement, Function, FunctionBody, StaticBlock,
    SwitchStatement, VariableDeclaration,
};
use oxc_ast_visit::{walk, Visit};
use oxc_syntax::scope::ScopeFlags;

impl<'a> Visit<'a> for NextjsCachingVisitor<'a> {
    fn visit_import_declaration(&mut self, import: &oxc_ast::ast::ImportDeclaration<'a>) {
        self.check_import(import);
        walk::walk_import_declaration(self, import);
    }

    fn visit_export_named_declaration(
        &mut self,
        export: &oxc_ast::ast::ExportNamedDeclaration<'a>,
    ) {
        self.check_export_specifiers(export);
        walk::walk_export_named_declaration(self, export);
    }

    fn visit_export_declaration(&mut self, export: &oxc_ast::ast::ExportDeclaration<'a>) {
        self.check_export(export);
        walk::walk_export_declaration(self, export);
    }

    fn visit_export_default_declaration(
        &mut self,
        export: &oxc_ast::ast::ExportDefaultDeclaration<'a>,
    ) {
        self.check_default_export(export);
        walk::walk_export_default_declaration(self, export);
    }

    fn visit_function(&mut self, function: &Function<'a>, flags: ScopeFlags) {
        self.open_function_scope(
            &function.params,
            function.id.as_ref().map(|id| id.name.as_str()),
        );
        walk::walk_function(self, function, flags);
        self.pop_const_scope();
    }

    fn visit_arrow_function_expression(&mut self, arrow: &ArrowFunctionExpression<'a>) {
        self.open_function_scope(&arrow.params, None);
        walk::walk_arrow_function_expression(self, arrow);
        self.pop_const_scope();
    }

    fn visit_function_body(&mut self, body: &FunctionBody<'a>) {
        self.open_function_body_scope(body);
        self.check_function_body_directives(body);
        walk::walk_function_body(self, body);
        self.pop_const_scope();
    }

    fn visit_block_statement(&mut self, block: &BlockStatement<'a>) {
        self.push_const_scope();
        self.precollect_statements(&block.body);
        walk::walk_block_statement(self, block);
        self.pop_const_scope();
    }

    fn visit_variable_declaration(&mut self, declaration: &VariableDeclaration<'a>) {
        self.record_const_declaration(declaration);
        walk::walk_variable_declaration(self, declaration);
    }

    fn visit_catch_clause(&mut self, catch: &CatchClause<'a>) {
        self.open_catch_scope(catch);
        walk::walk_catch_clause(self, catch);
        self.pop_const_scope();
    }

    fn visit_class(&mut self, class: &Class<'a>) {
        self.open_class_scope(class);
        walk::walk_class(self, class);
        self.pop_const_scope();
    }

    fn visit_static_block(&mut self, block: &StaticBlock<'a>) {
        self.open_static_scope(block);
        walk::walk_static_block(self, block);
        self.pop_const_scope();
    }

    fn visit_for_statement(&mut self, statement: &ForStatement<'a>) {
        self.open_for_scope(statement);
        walk::walk_for_statement(self, statement);
        self.pop_const_scope();
    }

    fn visit_for_in_statement(&mut self, statement: &ForInStatement<'a>) {
        self.open_for_left_scope(&statement.left);
        walk::walk_for_in_statement(self, statement);
        self.pop_const_scope();
    }

    fn visit_for_of_statement(&mut self, statement: &ForOfStatement<'a>) {
        self.open_for_left_scope(&statement.left);
        walk::walk_for_of_statement(self, statement);
        self.pop_const_scope();
    }

    fn visit_switch_statement(&mut self, statement: &SwitchStatement<'a>) {
        // The discriminant is evaluated before the shared lexical environment of the cases.
        self.visit_expression(&statement.discriminant);
        self.open_switch_scope(statement);
        self.visit_switch_cases(&statement.cases);
        self.pop_const_scope();
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        self.check_call(call);
        self.check_fetch_call(call);
        walk::walk_call_expression(self, call);
    }

    fn visit_assignment_expression(&mut self, assignment: &AssignmentExpression<'a>) {
        self.check_assignment(assignment);
        walk::walk_assignment_expression(self, assignment);
    }
}

use super::visitor::NextjsCachingVisitor;
use oxc_ast::ast::{
    AssignmentExpression, BlockStatement, CallExpression, FunctionBody, VariableDeclaration,
};
use oxc_ast_visit::{walk, Visit};

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

    fn visit_function_body(&mut self, body: &FunctionBody<'a>) {
        self.check_function_body_directives(body);
        self.push_const_scope();
        walk::walk_function_body(self, body);
        self.pop_const_scope();
    }

    fn visit_block_statement(&mut self, block: &BlockStatement<'a>) {
        self.push_const_scope();
        walk::walk_block_statement(self, block);
        self.pop_const_scope();
    }

    fn visit_variable_declaration(&mut self, declaration: &VariableDeclaration<'a>) {
        self.record_const_declaration(declaration);
        walk::walk_variable_declaration(self, declaration);
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

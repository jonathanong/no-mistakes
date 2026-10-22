use super::visitor::NextjsCachingVisitor;
use oxc_ast::ast::{
    ArrowFunctionBody, ArrowFunctionExpression, AssignmentExpression, BlockStatement,
    CallExpression, Function, FunctionBody, Statement, VariableDeclaration,
};
use oxc_ast_visit::{walk, Visit};
use oxc_syntax::scope::ScopeFlags;

fn arrow_statements<'a, 'b>(body: &'b ArrowFunctionBody<'a>) -> Option<&'b [Statement<'a>]> {
    match body {
        ArrowFunctionBody::FunctionBody(body) => Some(body.statements.as_slice()),
        _ => None,
    }
}

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
        let statements = function
            .body
            .as_ref()
            .map(|body| body.statements.as_slice());
        self.open_function_scope(&function.params, statements);
        walk::walk_function(self, function, flags);
        self.pop_const_scope();
    }

    fn visit_arrow_function_expression(&mut self, arrow: &ArrowFunctionExpression<'a>) {
        self.open_function_scope(&arrow.params, arrow_statements(&arrow.body));
        walk::walk_arrow_function_expression(self, arrow);
        self.pop_const_scope();
    }

    fn visit_function_body(&mut self, body: &FunctionBody<'a>) {
        self.check_function_body_directives(body);
        walk::walk_function_body(self, body);
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

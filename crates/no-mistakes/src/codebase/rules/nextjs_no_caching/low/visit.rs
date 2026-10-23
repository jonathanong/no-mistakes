use super::{Collector, NextjsCachingFinding};
use oxc_ast::ast::*;
use std::path::Path;

pub(crate) fn extract_program(
    path: &Path,
    source: &str,
    program: &Program<'_>,
) -> Vec<NextjsCachingFinding> {
    use oxc_ast_visit::Visit;
    let mut collector = Collector::new(path, source, program);
    collector.visit_program(program);
    collector.finish()
}
impl<'a> oxc_ast_visit::Visit<'a> for Collector<'a> {
    fn visit_import_declaration(&mut self, node: &ImportDeclaration<'a>) {
        self.import_declaration(node);
        oxc_ast_visit::walk::walk_import_declaration(self, node);
    }
    fn visit_call_expression(&mut self, node: &CallExpression<'a>) {
        self.call_expression(node);
        oxc_ast_visit::walk::walk_call_expression(self, node);
    }
    fn visit_export_declaration(&mut self, node: &ExportDeclaration<'a>) {
        self.export_declaration(node);
        oxc_ast_visit::walk::walk_export_declaration(self, node);
    }
    fn visit_export_named_declaration(&mut self, node: &ExportNamedDeclaration<'a>) {
        self.export_named_declaration(node);
        oxc_ast_visit::walk::walk_export_named_declaration(self, node);
    }
    fn visit_export_default_declaration(&mut self, node: &ExportDefaultDeclaration<'a>) {
        self.export_default_declaration(node);
        oxc_ast_visit::walk::walk_export_default_declaration(self, node);
    }
    fn visit_assignment_expression(&mut self, node: &AssignmentExpression<'a>) {
        self.assignment_expression(node);
        oxc_ast_visit::walk::walk_assignment_expression(self, node);
    }
    fn visit_function_body(&mut self, node: &FunctionBody<'a>) {
        self.function_body(node);
        oxc_ast_visit::walk::walk_function_body(self, node);
    }
}

use super::{binding_name, calls, module_export_name, AnalysisCollector};
use oxc_ast::ast::Statement;
use oxc_ast_visit::{walk, Visit};
use oxc_span::GetSpan;

impl<'a> Visit<'a> for AnalysisCollector<'a, '_> {
    fn visit_import_declaration(&mut self, import: &oxc_ast::ast::ImportDeclaration<'a>) {
        let source = import.source.value.to_string();
        if let Some(specifiers) = import.specifiers.as_ref() {
            for specifier in specifiers {
                self.collect_import(specifier, &source);
            }
        }
    }

    fn visit_statement(&mut self, statement: &Statement<'a>) {
        record_discovery_visit!();
        if self.statement_depth == 0 {
            match statement {
                Statement::FunctionDeclaration(function) => {
                    if let Some(id) = &function.id {
                        self.collect_function(
                            id.name.as_str(),
                            function.span(),
                            calls::collect_function_calls(function),
                        );
                    }
                }
                Statement::VariableDeclaration(declaration) => {
                    for declarator in &declaration.declarations {
                        let Some(name) = binding_name(&declarator.id) else {
                            continue;
                        };
                        self.collect_init_function(name, declarator.init.as_ref());
                    }
                }
                Statement::ExportNamedDeclaration(export) => {
                    for specifier in &export.specifiers {
                        self.result.exports.insert(
                            module_export_name(&specifier.exported),
                            module_export_name(&specifier.local),
                        );
                    }
                }
                Statement::ExportDeclaration(export) => {
                    self.collect_export_declaration(&export.declaration);
                }
                Statement::ExportDefaultDeclaration(export) => self.collect_default_export(export),
                _ => {}
            }
        }
        if !self.collect_tests {
            if let Statement::ImportDeclaration(import) = statement {
                self.visit_import_declaration(import);
            }
            return;
        }
        self.statement_depth += 1;
        walk::walk_statement(self, statement);
        self.statement_depth -= 1;
    }

    fn visit_call_expression(&mut self, call: &oxc_ast::ast::CallExpression<'a>) {
        record_discovery_visit!();
        if calls::is_skipped_describe(call) {
            return;
        }
        if let Some(name) = calls::describe_name(call) {
            self.describe_stack.push(name);
            walk::walk_call_expression(self, call);
            self.describe_stack.pop();
            return;
        }
        if let Some(test_name) = calls::test_name(call) {
            if let Some((argument, span)) = calls::callback_argument(call) {
                self.collect_test(call.span.start, test_name, argument, span);
                return;
            }
        }
        walk::walk_call_expression(self, call);
    }
}

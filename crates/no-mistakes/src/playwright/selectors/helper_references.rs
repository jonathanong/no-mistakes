use crate::codebase::ts_source::byte_offset_to_line;
use crate::playwright::{ast, playwright_tests};
use oxc_ast_visit::Visit;

mod arguments;
mod classifier;
use arguments::helper_argument_literals;
use classifier::is_helper_reference_call;

pub fn extract_playwright_helper_reference_occurrences_from_program(
    program: &oxc_ast::ast::Program<'_>,
    source: &str,
) -> Vec<playwright_tests::TestOccurrence<super::types::PlaywrightHelperReference>> {
    let mut visitor = PlaywrightHelperReferenceVisitor {
        source,
        status: playwright_tests::TestStatus::Active,
        annotation_status: playwright_tests::TestStatus::Active,
        references: Vec::new(),
        current_test_name: None,
        current_scope: playwright_tests::TestOccurrenceScope::File,
        describe_stack: Vec::new(),
    };
    visitor.visit_program(program);
    playwright_tests::dedup_occurrences_by_identity(&mut visitor.references);
    visitor.references
}

struct PlaywrightHelperReferenceVisitor<'a> {
    source: &'a str,
    status: playwright_tests::TestStatus,
    annotation_status: playwright_tests::TestStatus,
    references: Vec<playwright_tests::TestOccurrence<super::types::PlaywrightHelperReference>>,
    current_test_name: Option<String>,
    current_scope: playwright_tests::TestOccurrenceScope,
    describe_stack: Vec<String>,
}

impl<'a> oxc_ast_visit::Visit<'a> for PlaywrightHelperReferenceVisitor<'a> {
    fn visit_call_expression(&mut self, call: &oxc_ast::ast::CallExpression<'a>) {
        if let Some(path) = ast::expression_path(&call.callee) {
            if is_helper_reference_call(&call.callee, &path) {
                let call_display = format!("{}(...)", path.join("."));
                for value in helper_argument_literals(call, self.source) {
                    self.insert(value, call_display.clone(), call.span.start);
                }
            }
        }

        let traversal = playwright_tests::test_callback_traversal(call, self.annotation_status);
        if let Some((callback_index, callback_status)) = traversal {
            if let Some(describe) = playwright_tests::describe_name(call) {
                self.describe_stack.push(describe);
                for (index, argument) in call.arguments.iter().enumerate() {
                    if index == callback_index {
                        self.with_status(callback_status, |visitor| {
                            visitor
                                .with_annotation_scope(|visitor| visitor.visit_argument(argument));
                        });
                    } else {
                        self.visit_argument(argument);
                    }
                }
                self.describe_stack.pop();
            } else {
                let test_name = playwright_tests::test_callback_identity(call);
                let previous_test_name = self.current_test_name.clone();
                let previous_scope = self.current_scope;
                self.current_test_name = test_name;
                self.current_scope = playwright_tests::TestOccurrenceScope::Test;
                for (index, argument) in call.arguments.iter().enumerate() {
                    if index == callback_index {
                        self.with_status(callback_status, |visitor| {
                            visitor
                                .with_annotation_scope(|visitor| visitor.visit_argument(argument));
                        });
                    } else {
                        self.visit_argument(argument);
                    }
                }
                self.current_test_name = previous_test_name;
                self.current_scope = previous_scope;
            }
        } else {
            let callback_index = playwright_tests::callback_argument_index(call);
            if playwright_tests::annotation_status_for_call(call).is_some() {
                self.apply_annotation_call(call);
                for (index, argument) in call.arguments.iter().enumerate() {
                    if Some(index) != callback_index {
                        self.visit_argument(argument);
                    }
                }
                return;
            }
            if let Some((callback_index, hook_kind)) = playwright_tests::hook_callback(call) {
                let previous_scope = self.current_scope;
                self.current_scope = match hook_kind {
                    playwright_tests::HookKind::Setup => {
                        playwright_tests::TestOccurrenceScope::Hook
                    }
                    playwright_tests::HookKind::Teardown => {
                        playwright_tests::TestOccurrenceScope::TeardownHook
                    }
                };
                self.visit_argument(&call.arguments[callback_index]);
                self.current_scope = previous_scope;
                return;
            }
            oxc_ast_visit::walk::walk_call_expression(self, call);
        }
    }

    fn visit_if_statement(&mut self, statement: &oxc_ast::ast::IfStatement<'a>) {
        self.visit_expression(&statement.test);
        let status = playwright_tests::status_for_if_branch(self.status);
        self.with_status(status, |visitor| {
            visitor.visit_statement(&statement.consequent);
            if let Some(alternate) = &statement.alternate {
                visitor.visit_statement(alternate);
            }
        });
    }

    fn visit_conditional_expression(
        &mut self,
        expression: &oxc_ast::ast::ConditionalExpression<'a>,
    ) {
        self.visit_expression(&expression.test);
        let status = playwright_tests::status_for_if_branch(self.status);
        self.with_status(status, |visitor| {
            visitor.visit_expression(&expression.consequent);
            visitor.visit_expression(&expression.alternate);
        });
    }

    fn visit_logical_expression(&mut self, expression: &oxc_ast::ast::LogicalExpression<'a>) {
        self.visit_expression(&expression.left);
        let status = playwright_tests::status_for_if_branch(self.status);
        self.with_status(status, |visitor| {
            visitor.visit_expression(&expression.right)
        });
    }
}

impl PlaywrightHelperReferenceVisitor<'_> {
    fn insert(&mut self, value: String, call: String, byte_offset: u32) {
        self.references.push(playwright_tests::TestOccurrence {
            value: super::types::PlaywrightHelperReference { value, call },
            status: self.status.merge(self.annotation_status),
            scope: self.current_scope,
            test_name: self.current_test_name.clone(),
            describe_path: self.describe_stack.clone(),
            line: byte_offset_to_line(self.source, byte_offset as usize),
        });
    }

    fn with_status(&mut self, status: playwright_tests::TestStatus, visit: impl FnOnce(&mut Self)) {
        let previous = self.status;
        self.status = previous.merge(status);
        visit(self);
        self.status = previous;
    }

    fn with_annotation_scope(&mut self, visit: impl FnOnce(&mut Self)) {
        let previous = self.annotation_status;
        self.annotation_status = playwright_tests::TestStatus::Active;
        visit(self);
        self.annotation_status = previous;
    }

    fn apply_annotation_call(&mut self, call: &oxc_ast::ast::CallExpression<'_>) {
        if let Some(status) = playwright_tests::annotation_status_for_call(call) {
            let status = playwright_tests::merge_annotation_status(self.status, status);
            self.annotation_status =
                playwright_tests::merge_annotation_status(self.annotation_status, status);
        }
    }
}

#[cfg(test)]
mod tests;

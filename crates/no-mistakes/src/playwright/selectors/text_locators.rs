use super::text_locator_calls::extract_text_locator_call;
use crate::codebase::ts_source::byte_offset_to_line;
use crate::playwright::analysis::text_types::PlaywrightTextLocator;
use crate::playwright::playwright_tests;
use oxc_ast::ast::CallExpression;
use oxc_ast_visit::Visit;

pub(crate) fn extract_playwright_text_locator_occurrences_from_program(
    program: &oxc_ast::ast::Program<'_>,
    source: &str,
) -> Vec<playwright_tests::TestOccurrence<PlaywrightTextLocator>> {
    let mut visitor = PlaywrightTextLocatorVisitor {
        source,
        status: playwright_tests::TestStatus::Active,
        annotation_status: playwright_tests::TestStatus::Active,
        locators: Vec::new(),
        current_test_name: None,
        describe_stack: Vec::new(),
    };
    visitor.visit_program(program);
    playwright_tests::dedup_occurrences_by_identity(&mut visitor.locators);
    visitor.locators
}

struct PlaywrightTextLocatorVisitor<'a> {
    source: &'a str,
    status: playwright_tests::TestStatus,
    annotation_status: playwright_tests::TestStatus,
    locators: Vec<playwright_tests::TestOccurrence<PlaywrightTextLocator>>,
    current_test_name: Option<String>,
    describe_stack: Vec<String>,
}

impl<'a> Visit<'a> for PlaywrightTextLocatorVisitor<'_> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if let Some(locator) = extract_text_locator_call(call, self.source) {
            self.locators.push(playwright_tests::TestOccurrence {
                value: locator,
                status: self.status.merge(self.annotation_status),
                scope: playwright_tests::TestOccurrenceScope::Test,
                test_name: self.current_test_name.clone(),
                describe_path: self.describe_stack.clone(),
                line: byte_offset_to_line(self.source, call.span.start as usize),
            });
        }

        let traversal = playwright_tests::test_callback_traversal(call, self.annotation_status);
        if let Some((callback_index, callback_status)) = traversal {
            self.visit_playwright_callback(call, callback_index, callback_status);
            return;
        }

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
        oxc_ast_visit::walk::walk_call_expression(self, call);
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

impl PlaywrightTextLocatorVisitor<'_> {
    fn visit_playwright_callback(
        &mut self,
        call: &CallExpression<'_>,
        callback_index: usize,
        callback_status: playwright_tests::TestStatus,
    ) {
        if let Some(describe) = playwright_tests::describe_name(call) {
            self.describe_stack.push(describe);
            self.visit_callback_arguments(call, callback_index, callback_status);
            self.describe_stack.pop();
            return;
        }
        let test_name = playwright_tests::test_callback_identity(call);
        let previous_test_name = self.current_test_name.clone();
        if test_name.is_some() {
            self.current_test_name = test_name;
        }
        self.visit_callback_arguments(call, callback_index, callback_status);
        self.current_test_name = previous_test_name;
    }

    fn visit_callback_arguments(
        &mut self,
        call: &CallExpression<'_>,
        callback_index: usize,
        callback_status: playwright_tests::TestStatus,
    ) {
        for (index, argument) in call.arguments.iter().enumerate() {
            if index == callback_index {
                self.with_status(callback_status, |visitor| {
                    visitor.with_annotation_scope(|visitor| visitor.visit_argument(argument));
                });
            } else {
                self.visit_argument(argument);
            }
        }
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

    fn apply_annotation_call(&mut self, call: &CallExpression<'_>) {
        if let Some(status) = playwright_tests::annotation_status_for_call(call) {
            let status = playwright_tests::merge_annotation_status(self.status, status);
            self.annotation_status =
                playwright_tests::merge_annotation_status(self.annotation_status, status);
        }
    }
}

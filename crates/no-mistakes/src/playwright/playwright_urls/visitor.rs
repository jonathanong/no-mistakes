use crate::playwright::playwright_tests;
use oxc_ast::ast::{
    CallExpression, ConditionalExpression, IfStatement, LogicalExpression, Program,
};
use oxc_ast_visit::{walk, Visit};
use std::collections::HashMap;

mod calls;
mod context;

pub(super) struct UrlVisitor<'a, 'h> {
    pub source: &'a str,
    pub navigation_helpers: &'h [String],
    pub static_zero_arg_paths: &'h HashMap<String, Vec<String>>,
    pub status: playwright_tests::TestStatus,
    pub annotation_status: playwright_tests::TestStatus,
    pub urls: Vec<playwright_tests::TestOccurrence<String>>,
    pub current_test_name: Option<String>,
    pub current_scope: playwright_tests::TestOccurrenceScope,
    pub describe_stack: Vec<String>,
}

impl<'a> Visit<'a> for UrlVisitor<'a, '_> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        self.collect_call_urls(call);
        if self.visit_test_callback_call(call) || self.visit_hook_callback_call(call) {
            return;
        }
        if self.visit_annotation_call(call) {
            return;
        }
        walk::walk_call_expression(self, call);
    }

    fn visit_if_statement(&mut self, statement: &IfStatement<'a>) {
        self.visit_expression(&statement.test);
        let status = playwright_tests::status_for_if_branch(self.status);
        self.with_status(status, |visitor| {
            visitor.visit_statement(&statement.consequent);
            if let Some(alternate) = &statement.alternate {
                visitor.visit_statement(alternate);
            }
        });
    }

    fn visit_conditional_expression(&mut self, expression: &ConditionalExpression<'a>) {
        self.visit_expression(&expression.test);
        let status = playwright_tests::status_for_if_branch(self.status);
        self.with_status(status, |visitor| {
            visitor.visit_expression(&expression.consequent);
            visitor.visit_expression(&expression.alternate);
        });
    }

    fn visit_logical_expression(&mut self, expression: &LogicalExpression<'a>) {
        self.visit_expression(&expression.left);
        let status = playwright_tests::status_for_if_branch(self.status);
        self.with_status(status, |visitor| {
            visitor.visit_expression(&expression.right)
        });
    }
}

pub fn extract_playwright_url_occurrences_from_program(
    program: &Program<'_>,
    source: &str,
    navigation_helpers: &[String],
) -> Vec<playwright_tests::TestOccurrence<String>> {
    use super::statics::collect_static_zero_arg_paths;
    use oxc_ast_visit::Visit as _;

    let static_zero_arg_paths = collect_static_zero_arg_paths(source);
    let mut visitor = UrlVisitor {
        source,
        navigation_helpers,
        static_zero_arg_paths: &static_zero_arg_paths,
        status: playwright_tests::TestStatus::Active,
        annotation_status: playwright_tests::TestStatus::Active,
        urls: Vec::new(),
        current_test_name: None,
        current_scope: playwright_tests::TestOccurrenceScope::File,
        describe_stack: Vec::new(),
    };
    visitor.visit_program(program);
    playwright_tests::dedup_occurrences_by_identity(&mut visitor.urls);
    visitor.urls
}

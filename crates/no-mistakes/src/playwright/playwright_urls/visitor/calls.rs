use super::UrlVisitor;
use crate::playwright::{ast, playwright_tests};
use oxc_ast::ast::{Argument, CallExpression};
use oxc_ast_visit::Visit;

use crate::playwright::playwright_urls::callee::{
    callee_has_not, callee_is_member_named, callee_is_page_url_to_match,
    callee_is_playwright_wait_for_url, callee_matches_navigation_helper, is_candidate_url,
};
use crate::playwright::playwright_urls::literals::{
    argument_candidate_literals, argument_literals, direct_url_pattern_literals,
    extract_href_from_selector,
};

impl<'a, 'h> UrlVisitor<'a, 'h> {
    pub(super) fn collect_call_urls(&mut self, call: &CallExpression<'a>) {
        let callee = ast::expression_path(&call.callee);

        if callee_is_member_named(&call.callee, "goto") {
            self.collect_goto_urls(call);
        } else if callee_is_member_named(&call.callee, "click") {
            self.collect_click_url(call);
        } else if self.is_url_assertion_call(call, &callee) {
            self.collect_direct_url_patterns(call);
        } else if callee_matches_navigation_helper(&callee, self.navigation_helpers) {
            self.collect_navigation_helper_urls(call);
        }
    }

    fn collect_goto_urls(&mut self, call: &CallExpression<'a>) {
        let Some(argument) = call.arguments.first() else {
            return;
        };
        for url in argument_literals(argument, self.source, self.static_zero_arg_paths) {
            if is_candidate_url(&url) {
                self.insert(url, call.span.start);
            }
        }
    }

    fn collect_click_url(&mut self, call: &CallExpression<'a>) {
        let Some(argument) = call.arguments.first() else {
            return;
        };
        for selector in argument_literals(argument, self.source, self.static_zero_arg_paths) {
            if let Some(url) = extract_href_from_selector(&selector) {
                self.insert(url, call.span.start);
            }
        }
    }

    fn is_url_assertion_call(
        &self,
        call: &CallExpression<'a>,
        callee: &Option<Vec<String>>,
    ) -> bool {
        !callee_has_not(callee)
            && (callee_is_member_named(&call.callee, "toHaveURL")
                || callee_is_playwright_wait_for_url(&call.callee)
                || callee_is_page_url_to_match(&call.callee))
    }

    fn collect_direct_url_patterns(&mut self, call: &CallExpression<'a>) {
        for url in
            direct_url_pattern_literals(&call.arguments, self.source, self.static_zero_arg_paths)
        {
            self.insert(url, call.span.start);
        }
    }

    fn collect_navigation_helper_urls(&mut self, call: &CallExpression<'a>) {
        for argument in &call.arguments {
            let urls =
                argument_candidate_literals(argument, self.source, self.static_zero_arg_paths);
            if urls.is_empty() {
                continue;
            }
            for url in urls {
                self.insert(url, call.span.start);
            }
            break;
        }
    }

    pub(super) fn visit_test_callback_call(&mut self, call: &CallExpression<'a>) -> bool {
        let Some((callback_index, callback_status)) =
            playwright_tests::test_callback_traversal(call, self.annotation_status)
        else {
            return false;
        };

        if let Some(describe) = playwright_tests::describe_name(call) {
            self.describe_stack.push(describe);
            self.visit_callback_arguments(call, callback_index, callback_status);
            self.describe_stack.pop();
            return true;
        }

        let test_name = playwright_tests::test_callback_identity(call);
        let previous_test_name = self.current_test_name.clone();
        let previous_scope = self.current_scope;
        if test_name.is_some() {
            self.current_test_name = test_name;
            self.current_scope = playwright_tests::TestOccurrenceScope::Test;
        }
        self.visit_callback_arguments(call, callback_index, callback_status);
        self.current_test_name = previous_test_name;
        self.current_scope = previous_scope;
        true
    }

    fn visit_callback_arguments(
        &mut self,
        call: &CallExpression<'a>,
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

    pub(super) fn visit_hook_callback_call(&mut self, call: &CallExpression<'a>) -> bool {
        let Some(callback_index) = playwright_tests::hook_callback_index(call) else {
            return false;
        };
        for (index, argument) in call.arguments.iter().enumerate() {
            if index == callback_index {
                self.with_scope(playwright_tests::TestOccurrenceScope::Hook, |visitor| {
                    visitor.visit_argument(argument)
                });
            } else {
                self.visit_argument(argument);
            }
        }
        true
    }

    pub(super) fn visit_annotation_call(&mut self, call: &CallExpression<'a>) -> bool {
        if playwright_tests::annotation_status_for_call(call).is_none() {
            return false;
        }
        let callback_index = playwright_tests::callback_argument_index(call);
        self.apply_annotation_call(call);
        self.visit_non_callback_arguments(&call.arguments, callback_index);
        true
    }

    fn visit_non_callback_arguments(
        &mut self,
        arguments: &[Argument<'a>],
        callback_index: Option<usize>,
    ) {
        for (index, argument) in arguments.iter().enumerate() {
            if Some(index) != callback_index {
                self.visit_argument(argument);
            }
        }
    }
}

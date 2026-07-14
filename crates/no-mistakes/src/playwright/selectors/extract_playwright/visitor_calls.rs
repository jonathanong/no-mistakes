use super::PlaywrightSelectorVisitor;
use crate::playwright::playwright_tests;
use crate::playwright::selectors::call_shapes::{
    callee_is_static_member_named, extract_get_by_test_id_call, extract_test_id_argument,
    selector_argument_literals, selector_argument_mode,
};
use crate::playwright::selectors::css::{
    extract_css_attribute_selectors, extract_css_id_selectors,
};
use oxc_ast_visit::Visit;

impl<'a> PlaywrightSelectorVisitor<'a, '_> {
    pub(super) fn extract_call_selectors(&mut self, call: &oxc_ast::ast::CallExpression<'a>) {
        if let Some(wrapper) = self
            .wrapper_bindings
            .call(&call.callee, &self.shadow_scopes)
        {
            self.wrapper_call_offsets.insert(call.span.start);
            if let Some(argument) = call.arguments.get(wrapper.test_id_argument) {
                extract_test_id_argument(
                    argument,
                    self.source,
                    self.test_id_attributes,
                    &wrapper.call_name,
                    &mut |selector| self.insert(selector, call.span.start),
                );
            }
        } else if self
            .wrapper_bindings
            .is_shadowed_configured_call(&call.callee, &self.shadow_scopes)
        {
        } else if callee_is_static_member_named(&call.callee, "getByTestId") {
            extract_get_by_test_id_call(
                call,
                self.source,
                self.test_id_attributes,
                &mut |selector| self.insert(selector, call.span.start),
            );
        } else if let Some(argument_mode) = selector_argument_mode(&call.callee) {
            for selector in selector_argument_literals(call, self.source, argument_mode) {
                extract_css_attribute_selectors(
                    &selector,
                    &self.regexes.playwright_attributes,
                    &mut |selector| self.insert(selector, call.span.start),
                );
                if self.regexes.html_ids {
                    extract_css_id_selectors(&selector, &mut |selector| {
                        self.insert(selector, call.span.start)
                    });
                }
            }
        }
    }

    pub(super) fn traverse_call(&mut self, call: &oxc_ast::ast::CallExpression<'a>) {
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
                let previous_test_name = self.current_test_name.clone();
                let previous_scope = self.current_scope;
                self.current_test_name = playwright_tests::test_callback_identity(call);
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
        if let Some((callback_index, hook_kind)) = playwright_tests::hook_callback(call) {
            let previous_scope = self.current_scope;
            self.current_scope = match hook_kind {
                playwright_tests::HookKind::Setup => playwright_tests::TestOccurrenceScope::Hook,
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

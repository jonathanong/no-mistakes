use super::{PlaywrightSelectorVisitor, WrapperBindings};
use crate::codebase::ts_source::byte_offset_to_line;
use crate::playwright::playwright_tests;
use crate::playwright::selectors::types::{PlaywrightSelector, SelectorRegexes};
use std::collections::HashSet;

impl<'a, 'r> PlaywrightSelectorVisitor<'a, 'r> {
    pub(super) fn new(
        source: &'a str,
        regexes: &'r SelectorRegexes,
        test_id_attributes: &'r [String],
        wrapper_bindings: WrapperBindings,
    ) -> Self {
        Self {
            source,
            regexes,
            test_id_attributes,
            status: playwright_tests::TestStatus::Active,
            annotation_status: playwright_tests::TestStatus::Active,
            selectors: Vec::new(),
            current_test_name: None,
            current_scope: playwright_tests::TestOccurrenceScope::File,
            describe_stack: Vec::new(),
            wrapper_bindings,
            wrapper_call_offsets: HashSet::new(),
            shadow_scopes: Vec::new(),
        }
    }

    pub(super) fn insert(&mut self, value: PlaywrightSelector, byte_offset: u32) {
        self.selectors.push(playwright_tests::TestOccurrence {
            value,
            status: self.status.merge(self.annotation_status),
            scope: self.current_scope,
            test_name: self.current_test_name.clone(),
            describe_path: self.describe_stack.clone(),
            line: byte_offset_to_line(self.source, byte_offset as usize),
        });
    }

    pub(super) fn with_status(
        &mut self,
        status: playwright_tests::TestStatus,
        visit: impl FnOnce(&mut Self),
    ) {
        let previous = self.status;
        self.status = previous.merge(status);
        visit(self);
        self.status = previous;
    }

    pub(super) fn with_annotation_scope(&mut self, visit: impl FnOnce(&mut Self)) {
        let previous = self.annotation_status;
        self.annotation_status = playwright_tests::TestStatus::Active;
        visit(self);
        self.annotation_status = previous;
    }

    pub(super) fn apply_annotation_call(&mut self, call: &oxc_ast::ast::CallExpression<'_>) {
        if let Some(status) = playwright_tests::annotation_status_for_call(call) {
            let status = playwright_tests::merge_annotation_status(self.status, status);
            self.annotation_status =
                playwright_tests::merge_annotation_status(self.annotation_status, status);
        }
    }

    pub(super) fn with_shadow_scope(
        &mut self,
        shadowed: HashSet<String>,
        visit: impl FnOnce(&mut Self),
    ) {
        self.shadow_scopes.push(shadowed);
        visit(self);
        self.shadow_scopes.pop();
    }
}

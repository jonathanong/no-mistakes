use super::UrlVisitor;
use crate::codebase::ts_source::byte_offset_to_line;
use crate::playwright::playwright_tests;
use oxc_ast::ast::CallExpression;

impl UrlVisitor<'_, '_> {
    pub(super) fn insert(&mut self, value: String, byte_offset: u32) {
        self.urls.push(playwright_tests::TestOccurrence {
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

    pub(super) fn with_scope(
        &mut self,
        scope: playwright_tests::TestOccurrenceScope,
        visit: impl FnOnce(&mut Self),
    ) {
        let previous = self.current_scope;
        self.current_scope = scope;
        visit(self);
        self.current_scope = previous;
    }

    pub(super) fn apply_annotation_call(&mut self, call: &CallExpression<'_>) {
        let status = playwright_tests::annotation_status_for_call(call)
            .expect("only annotation calls are applied");
        let status = playwright_tests::merge_annotation_status(self.status, status);
        self.annotation_status =
            playwright_tests::merge_annotation_status(self.annotation_status, status);
    }
}

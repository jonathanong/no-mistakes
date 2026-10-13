use crate::playwright::playwright_tests::{self, TestStatus};
use oxc_ast::ast::{ConditionalExpression, IfStatement, LogicalExpression};
use oxc_ast_visit::Visit;

pub(super) trait StatusAwareVisitor<'a>: Visit<'a> + Sized {
    fn current_status(&self) -> TestStatus;
    fn with_branch_status(&mut self, status: TestStatus, visit: impl FnOnce(&mut Self));

    fn visit_if_with_status(&mut self, statement: &IfStatement<'a>) {
        self.visit_expression(&statement.test);
        let status = playwright_tests::status_for_if_branch(self.current_status());
        self.with_branch_status(status, |visitor| {
            visitor.visit_statement(&statement.consequent);
            if let Some(alternate) = &statement.alternate {
                visitor.visit_statement(alternate);
            }
        });
    }

    fn visit_conditional_with_status(&mut self, expression: &ConditionalExpression<'a>) {
        self.visit_expression(&expression.test);
        let status = playwright_tests::status_for_if_branch(self.current_status());
        self.with_branch_status(status, |visitor| {
            visitor.visit_expression(&expression.consequent);
            visitor.visit_expression(&expression.alternate);
        });
    }

    fn visit_logical_with_status(&mut self, expression: &LogicalExpression<'a>) {
        self.visit_expression(&expression.left);
        let status = playwright_tests::status_for_if_branch(self.current_status());
        self.with_branch_status(status, |visitor| {
            visitor.visit_expression(&expression.right)
        });
    }
}

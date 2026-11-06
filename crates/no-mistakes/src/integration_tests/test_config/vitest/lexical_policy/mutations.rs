use oxc_ast::ast::{
    AssignmentExpression, Program, UnaryExpression, UnaryOperator, UpdateExpression,
};
use oxc_ast_visit::Visit;

#[derive(Default)]
struct Mutations(bool);
impl<'a> Visit<'a> for Mutations {
    fn visit_assignment_expression(&mut self, _: &AssignmentExpression<'a>) {
        self.0 = true;
    }
    fn visit_update_expression(&mut self, _: &UpdateExpression<'a>) {
        self.0 = true;
    }
    fn visit_unary_expression(&mut self, expression: &UnaryExpression<'a>) {
        if expression.operator == UnaryOperator::Delete {
            self.0 = true;
        }
        oxc_ast_visit::walk::walk_unary_expression(self, expression);
    }
}
pub(super) fn contains_mutation(program: &Program<'_>) -> bool {
    let mut visitor = Mutations::default();
    visitor.visit_program(program);
    visitor.0
}

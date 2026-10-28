use super::Uses;
use oxc_ast::ast;
use oxc_ast_visit::{walk, Visit};

// GuardedCase admits only a declaration, the throwing guard, a simple sink,
// and an optional return. Nested function/class visitors stop before statements.

impl<'a> Visit<'a> for Uses<'_> {
    fn enter_node(&mut self, _: oxc_ast::AstKind<'a>) {
        self.nodes += 1;
        if self.nodes > 512 {
            self.valid = false;
        }
    }
    fn visit_identifier_reference(&mut self, id: &ast::IdentifierReference<'a>) {
        if id.name == self.set && id.span.start != self.allowed_set
            || id.name == "Set" && id.span.start != self.allowed_constructor
            || matches!(id.name.as_str(), "eval" | "arguments")
        {
            self.valid = false;
        }
    }
    fn visit_binding_identifier(&mut self, id: &ast::BindingIdentifier<'a>) {
        if id.name == self.value
            || id.name == "Set"
            || id.name == self.set && id.span.start != self.set_declaration
        {
            self.valid = false;
        }
    }
    fn visit_assignment_expression(&mut self, _: &ast::AssignmentExpression<'a>) {
        self.valid = false;
    }
    fn visit_update_expression(&mut self, _: &ast::UpdateExpression<'a>) {
        self.valid = false;
    }
    fn visit_function(&mut self, _: &ast::Function<'a>, _: oxc_syntax::scope::ScopeFlags) {
        self.valid = false;
    }
    fn visit_arrow_function_expression(&mut self, _: &ast::ArrowFunctionExpression<'a>) {
        self.valid = false;
    }
    fn visit_class(&mut self, _: &ast::Class<'a>) {
        self.valid = false;
    }
    fn visit_unary_expression(&mut self, value: &ast::UnaryExpression<'a>) {
        if value.operator.is_delete() {
            self.valid = false;
        }
        walk::walk_unary_expression(self, value);
    }
}

use oxc_ast::ast::*;
use oxc_ast_visit::Visit;

#[derive(Default)]
pub(super) struct Assigned(pub(super) std::collections::HashSet<String>);
impl<'a> Visit<'a> for Assigned {
    fn visit_identifier_reference(&mut self, value: &IdentifierReference<'a>) {
        self.0.insert(value.name.to_string());
    }
    // Member receivers and default values are reads, not binding writes.
    fn visit_static_member_expression(&mut self, _: &StaticMemberExpression<'a>) {}
    fn visit_computed_member_expression(&mut self, _: &ComputedMemberExpression<'a>) {}
    fn visit_private_field_expression(&mut self, _: &PrivateFieldExpression<'a>) {}
    fn visit_expression(&mut self, _: &Expression<'a>) {}
}

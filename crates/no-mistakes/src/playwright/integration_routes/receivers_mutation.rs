/// Walk only assigned bindings/member roots, not computed keys or defaults.
struct MutationTargets<'p>(&'p mut Receivers);

impl<'a> Visit<'a> for MutationTargets<'_> {
    fn visit_identifier_reference(&mut self, identifier: &oxc_ast::ast::IdentifierReference<'a>) {
        self.0.invalid.insert(identifier.name.to_string());
    }
    fn visit_expression(&mut self, expression: &Expression<'a>) {
        self.0.invalidate_object(expression);
    }
    fn visit_static_member_expression(
        &mut self,
        member: &oxc_ast::ast::StaticMemberExpression<'a>,
    ) {
        self.0.invalidate_object(&member.object);
    }
    fn visit_computed_member_expression(
        &mut self,
        member: &oxc_ast::ast::ComputedMemberExpression<'a>,
    ) {
        self.0.invalidate_object(&member.object);
    }
    fn visit_assignment_target_property_identifier(
        &mut self,
        property: &oxc_ast::ast::AssignmentTargetPropertyIdentifier<'a>,
    ) {
        self.visit_identifier_reference(&property.binding);
    }
    fn visit_assignment_target_property_property(
        &mut self,
        property: &oxc_ast::ast::AssignmentTargetPropertyProperty<'a>,
    ) {
        self.visit_assignment_target_maybe_default(&property.binding);
    }
    fn visit_assignment_target_with_default(
        &mut self,
        target: &oxc_ast::ast::AssignmentTargetWithDefault<'a>,
    ) {
        self.visit_assignment_target(&target.binding);
    }
}

use super::Writes;
use oxc_ast::ast::{BindingIdentifier, Expression, FormalParameters, TSType};
use oxc_ast_visit::Visit;

pub(in crate::codebase::postgres::embedded::walk) fn parameter_effects(
    writes: &mut Writes,
    params: &FormalParameters<'_>,
) {
    struct Initializers<'a>(&'a mut Writes);
    impl<'a> Visit<'a> for Initializers<'_> {
        fn visit_binding_identifier(&mut self, _: &BindingIdentifier<'a>) {}
        fn visit_ts_type(&mut self, _: &TSType<'a>) {}
        fn visit_expression(&mut self, expression: &Expression<'a>) {
            self.0.visit_expression(expression);
        }
    }
    Initializers(writes).visit_formal_parameters(params);
}

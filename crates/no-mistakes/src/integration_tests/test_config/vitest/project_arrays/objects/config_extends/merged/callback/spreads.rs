use super::{unwrap_ts_wrappers, Expression, ObjectExpression};
use oxc_ast::ast::{ArrowFunctionExpression, Function, ObjectPropertyKind};
use oxc_ast_visit::{walk, Visit};

pub(super) fn proven_object(object: &ObjectExpression<'_>) -> bool {
    let mut proof = Proof(true);
    proof.visit_object_expression(object);
    proof.0
}

struct Proof(bool);
impl<'a> Visit<'a> for Proof {
    fn visit_object_expression(&mut self, object: &ObjectExpression<'a>) {
        for property in &object.properties {
            match property {
                ObjectPropertyKind::SpreadProperty(spread) => {
                    // A reference can name a mutable object, even with a const binding.
                    if !matches!(
                        unwrap_ts_wrappers(&spread.argument),
                        Expression::ObjectExpression(_)
                    ) {
                        self.0 = false;
                    }
                }
                ObjectPropertyKind::ObjectProperty(property) => {
                    if property.computed
                        && crate::integration_tests::test_config::shared::property_key_name(
                            &property.key,
                        )
                        .is_none()
                    {
                        self.0 = false;
                    }
                }
            }
        }
        walk::walk_object_expression(self, object);
    }
    // Deferred function bodies cannot overwrite the returned object's fields.
    fn visit_function(&mut self, _function: &Function<'a>, _flags: oxc_syntax::scope::ScopeFlags) {}
    fn visit_arrow_function_expression(&mut self, _arrow: &ArrowFunctionExpression<'a>) {}
}

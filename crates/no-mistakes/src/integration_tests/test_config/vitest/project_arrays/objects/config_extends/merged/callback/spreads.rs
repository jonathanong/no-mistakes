use super::{unwrap_ts_wrappers, Expression, ObjectExpression};
use oxc_ast::ast::{ArrowFunctionExpression, Function, IdentifierReference, ObjectPropertyKind};
use oxc_ast_visit::{walk, Visit};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn proven_object<'a>(
    object: &ObjectExpression<'a>,
    bindings: &BTreeMap<String, &'a Expression<'a>>,
    immutable: &BTreeMap<String, &'a Expression<'a>>,
) -> bool {
    let mut proof = Proof {
        safe: true,
        bindings,
        immutable,
        visiting: BTreeSet::new(),
    };
    proof.visit_object_expression(object);
    proof.safe
}

struct Proof<'a, 'b> {
    safe: bool,
    bindings: &'b BTreeMap<String, &'a Expression<'a>>,
    immutable: &'b BTreeMap<String, &'a Expression<'a>>,
    visiting: BTreeSet<String>,
}
impl<'a> Visit<'a> for Proof<'a, '_> {
    fn visit_object_expression(&mut self, object: &ObjectExpression<'a>) {
        for property in &object.properties {
            match property {
                ObjectPropertyKind::SpreadProperty(spread) => {
                    // A reference can name a mutable object, even with a const binding.
                    if !matches!(
                        unwrap_ts_wrappers(&spread.argument),
                        Expression::ObjectExpression(_)
                    ) {
                        self.safe = false;
                    }
                }
                ObjectPropertyKind::ObjectProperty(property) => {
                    if property.computed
                        && crate::integration_tests::test_config::shared::property_key_name(
                            &property.key,
                        )
                        .is_none()
                    {
                        self.safe = false;
                    }
                }
            }
        }
        walk::walk_object_expression(self, object);
    }
    fn visit_identifier_reference(&mut self, identifier: &IdentifierReference<'a>) {
        let name = identifier.name.as_str();
        if !self.bindings.contains_key(name) {
            return;
        }
        let Some(value) = self.immutable.get(name).copied() else {
            self.safe = false;
            return;
        };
        if !self.visiting.insert(name.to_string()) {
            self.safe = false;
            return;
        }
        self.visit_expression(value);
        self.visiting.remove(name);
    }
    // Deferred function bodies cannot overwrite the returned object's fields.
    fn visit_function(&mut self, _function: &Function<'a>, _flags: oxc_syntax::scope::ScopeFlags) {}
    fn visit_arrow_function_expression(&mut self, _arrow: &ArrowFunctionExpression<'a>) {}
}

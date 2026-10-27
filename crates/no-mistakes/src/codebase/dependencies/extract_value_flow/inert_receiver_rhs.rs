use super::*;
#[cfg(feature = "test-instrumentation")]
use oxc_span::GetSpan;

// This gate admits only supported inert construction syntax. It does not prove
// arbitrary JavaScript purity or evaluate getters, spreads, calls, or classes.
pub(super) fn inert_rhs(
    expression: &Expression<'_>,
    effects: &mut fresh_receiver_effects::FreshReceiverEffects,
) -> bool {
    #[cfg(feature = "test-instrumentation")]
    if effects.rhs_probe.as_ref().is_some_and(|p| p.borrow().old) {
        return tests_support::old_rhs(expression, effects);
    }
    if matches!(
        crate::codebase::ts_source::unwrap_ts_wrappers(expression),
        Expression::Identifier(_)
    ) {
        effects.note_rhs_visit();
        return false;
    }
    let mut pending = PendingExpressions::new(expression);
    while let Some(expression) = pending.pop() {
        effects.note_rhs_visit();
        #[cfg(feature = "test-instrumentation")]
        tests_support::visit(effects, "expression", expression.span());
        match crate::codebase::ts_source::unwrap_ts_wrappers(expression) {
            Expression::Identifier(_)
            | Expression::BooleanLiteral(_)
            | Expression::NullLiteral(_)
            | Expression::NumericLiteral(_)
            | Expression::BigIntLiteral(_)
            | Expression::StringLiteral(_)
            | Expression::RegExpLiteral(_)
            | Expression::FunctionExpression(_)
            | Expression::ArrowFunctionExpression(_) => {}
            Expression::ObjectExpression(object) => {
                for property in &object.properties {
                    effects.note_rhs_visit();
                    #[cfg(feature = "test-instrumentation")]
                    tests_support::visit(effects, "property", property.span());
                    let ObjectPropertyKind::ObjectProperty(property) = property else {
                        return false;
                    };
                    if property.computed
                        || property.kind != oxc_ast::ast::PropertyKind::Init
                        || crate::codebase::ts_source::static_property_key_name(&property.key)
                            .as_deref()
                            == Some("__proto__")
                    {
                        return false;
                    }
                    let grew = pending.push(&property.value);
                    #[cfg(feature = "test-instrumentation")]
                    tests_support::growth(effects, grew);
                    #[cfg(not(feature = "test-instrumentation"))]
                    let _ = grew;
                }
            }
            Expression::ArrayExpression(array) => {
                for element in &array.elements {
                    effects.note_rhs_visit();
                    #[cfg(feature = "test-instrumentation")]
                    tests_support::visit(effects, "element", element.span());
                    if matches!(element, oxc_ast::ast::ArrayExpressionElement::Elision(_)) {
                        continue;
                    }
                    let Some(value) = element.as_expression() else {
                        return false;
                    };
                    let grew = pending.push(value);
                    #[cfg(feature = "test-instrumentation")]
                    tests_support::growth(effects, grew);
                    #[cfg(not(feature = "test-instrumentation"))]
                    let _ = grew;
                }
            }
            _ => return false,
        }
    }
    true
}

// The logical stack is spill followed by the optional inline top. Allocate only
// when a second expression must remain pending; preserve the old Vec LIFO order.
struct PendingExpressions<'expression, 'ast> {
    top: Option<&'expression Expression<'ast>>,
    spill: Vec<&'expression Expression<'ast>>,
}

impl<'expression, 'ast> PendingExpressions<'expression, 'ast> {
    fn new(expression: &'expression Expression<'ast>) -> Self {
        Self {
            top: Some(expression),
            spill: Vec::new(),
        }
    }

    fn push(&mut self, expression: &'expression Expression<'ast>) -> bool {
        #[cfg(feature = "test-instrumentation")]
        let capacity = self.spill.capacity();
        if let Some(previous) = self.top.replace(expression) {
            self.spill.push(previous);
        }
        #[cfg(feature = "test-instrumentation")]
        {
            self.spill.capacity() != capacity
        }
        #[cfg(not(feature = "test-instrumentation"))]
        {
            false
        }
    }

    fn pop(&mut self) -> Option<&'expression Expression<'ast>> {
        self.top.take().or_else(|| self.spill.pop())
    }
}

#[cfg(all(test, feature = "test-instrumentation"))]
#[path = "inert_rhs_tests.rs"]
mod tests;
#[cfg(feature = "test-instrumentation")]
#[path = "inert_rhs_probe.rs"]
pub(in crate::codebase::dependencies) mod tests_support;

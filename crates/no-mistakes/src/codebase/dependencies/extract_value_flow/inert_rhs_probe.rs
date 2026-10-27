use super::*;
#[derive(Default, Debug)]
pub(in crate::codebase::dependencies) struct Probe {
    pub old: bool,
    pub old_allocations: usize,
    pub spill_growths: usize,
    pub trace: Vec<(&'static str, u32, u32)>,
}
pub(super) fn visit(
    effects: &mut fresh_receiver_effects::FreshReceiverEffects,
    kind: &'static str,
    span: oxc_span::Span,
) {
    if let Some(probe) = &effects.rhs_probe {
        probe.borrow_mut().trace.push((kind, span.start, span.end));
    }
}
pub(super) fn growth(effects: &mut fresh_receiver_effects::FreshReceiverEffects, grew: bool) {
    if let Some(probe) = &effects.rhs_probe {
        probe.borrow_mut().spill_growths += usize::from(grew);
    }
}

// This gate admits only supported inert construction syntax. It does not prove
// arbitrary JavaScript purity or evaluate getters, spreads, calls, or classes.
pub(super) fn old_rhs(
    expression: &Expression<'_>,
    effects: &mut fresh_receiver_effects::FreshReceiverEffects,
) -> bool {
    if matches!(
        crate::codebase::ts_source::unwrap_ts_wrappers(expression),
        Expression::Identifier(_)
    ) {
        effects.note_rhs_visit();
        return false;
    }
    if let Some(probe) = &effects.rhs_probe {
        probe.borrow_mut().old_allocations += 1;
    }
    let mut pending = vec![expression];
    while let Some(expression) = pending.pop() {
        effects.note_rhs_visit();
        visit(effects, "expression", expression.span());
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
                    visit(effects, "property", property.span());
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
                    pending.push(&property.value);
                }
            }
            Expression::ArrayExpression(array) => {
                for element in &array.elements {
                    effects.note_rhs_visit();
                    visit(effects, "element", element.span());
                    if matches!(element, oxc_ast::ast::ArrayExpressionElement::Elision(_)) {
                        continue;
                    }
                    let Some(value) = element.as_expression() else {
                        return false;
                    };
                    pending.push(value);
                }
            }
            _ => return false,
        }
    }
    true
}

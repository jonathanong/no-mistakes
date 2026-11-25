use crate::fx::FxHashSet;
use crate::queue::object_bindings::ObjectBindings;
use oxc_ast::ast::{Expression, ObjectExpression, ObjectPropertyKind};
use oxc_span::GetSpan;

pub(in crate::queue) fn extract_member_from_expr<'a, 'b>(
    expr: &'b Expression<'a>,
    at: u32,
    key: &str,
    bindings: &'b ObjectBindings<'a>,
    active: &mut FxHashSet<u32>,
) -> Option<&'b Expression<'a>>
where
    'a: 'b,
{
    match expr.get_inner_expression() {
        Expression::ObjectExpression(object) => extract_member(object, key, bindings, active),
        Expression::Identifier(id) => {
            let binding = bindings.lookup(id.name.as_str(), at)?;
            let value = binding.value()?;
            if !active.insert(binding.resolve_at) {
                return None;
            }
            let found = extract_member_from_expr(value, binding.resolve_at, key, bindings, active);
            active.remove(&binding.resolve_at);
            found
        }
        _ => None,
    }
}

pub(in crate::queue) fn extract_member<'a, 'b>(
    object: &'b ObjectExpression<'a>,
    key: &str,
    bindings: &'b ObjectBindings<'a>,
    active: &mut FxHashSet<u32>,
) -> Option<&'b Expression<'a>>
where
    'a: 'b,
{
    let mut found = None;
    for property in &object.properties {
        match property {
            ObjectPropertyKind::ObjectProperty(property) => {
                if crate::ast::property_key_name(&property.key).as_deref() == Some(key) {
                    found = Some(&property.value);
                }
            }
            ObjectPropertyKind::SpreadProperty(spread) => {
                if let Some(value) = extract_member_from_expr(
                    &spread.argument,
                    spread.argument.span().start,
                    key,
                    bindings,
                    active,
                ) {
                    found = Some(value);
                }
            }
        }
    }
    found
}

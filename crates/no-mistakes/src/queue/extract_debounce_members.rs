use crate::fx::FxHashSet;
use crate::queue::object_bindings::ObjectBindings;
use oxc_ast::ast::{Expression, ObjectExpression, ObjectPropertyKind};
use oxc_span::GetSpan;

#[derive(Clone, Copy)]
pub(in crate::queue) enum MemberLookup<'ast, 'borrow> {
    Absent,
    Unknown,
    Found(&'borrow Expression<'ast>),
}

pub(in crate::queue) fn extract_member_from_expr<'ast, 'borrow>(
    expr: &'borrow Expression<'ast>,
    at: u32,
    key: &str,
    bindings: &'borrow ObjectBindings<'ast>,
    active: &mut FxHashSet<u32>,
) -> MemberLookup<'ast, 'borrow>
where
    'ast: 'borrow,
{
    match expr.get_inner_expression() {
        Expression::ObjectExpression(object) => extract_member(object, key, bindings, active),
        Expression::Identifier(id) => {
            let Some(binding) = bindings.lookup(id.name.as_str(), at) else {
                return MemberLookup::Unknown;
            };
            let Some(value) = binding.value() else {
                return MemberLookup::Unknown;
            };
            if !active.insert(binding.resolve_at) {
                return MemberLookup::Unknown;
            }
            let found = extract_member_from_expr(value, binding.resolve_at, key, bindings, active);
            active.remove(&binding.resolve_at);
            found
        }
        _ => MemberLookup::Unknown,
    }
}

pub(in crate::queue) fn extract_member<'ast, 'borrow>(
    object: &'borrow ObjectExpression<'ast>,
    key: &str,
    bindings: &'borrow ObjectBindings<'ast>,
    active: &mut FxHashSet<u32>,
) -> MemberLookup<'ast, 'borrow>
where
    'ast: 'borrow,
{
    let mut found = None;
    let mut unknown = false;
    for property in &object.properties {
        match property {
            ObjectPropertyKind::ObjectProperty(property) => {
                if crate::ast::property_key_name(&property.key).as_deref() == Some(key) {
                    found = Some(&property.value);
                    unknown = false;
                }
            }
            ObjectPropertyKind::SpreadProperty(spread) => {
                match extract_member_from_expr(
                    &spread.argument,
                    spread.argument.span().start,
                    key,
                    bindings,
                    active,
                ) {
                    MemberLookup::Found(value) => {
                        found = Some(value);
                        unknown = false;
                    }
                    MemberLookup::Absent => {}
                    MemberLookup::Unknown => unknown = true,
                }
            }
        }
    }
    if unknown {
        MemberLookup::Unknown
    } else if let Some(value) = found {
        MemberLookup::Found(value)
    } else {
        MemberLookup::Absent
    }
}

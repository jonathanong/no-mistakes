use super::names::{Names, ValueKind};
use oxc_ast::ast::*;

pub(super) fn concrete_type(ty: &TSType<'_>) -> bool {
    match ty {
        TSType::TSAnyKeyword(_) | TSType::TSUnknownKeyword(_) => false,
        TSType::TSParenthesizedType(inner) => concrete_type(&inner.type_annotation),
        _ => true,
    }
}

pub(super) fn record_custom(
    names: &mut Names,
    name: &str,
    kind: Option<ValueKind>,
    ty: Option<&TSType<'_>>,
    init: Option<&Expression<'_>>,
) {
    if kind.is_some() {
        return;
    }
    if ty.is_some_and(concrete_type)
        || init.and_then(cast_type).is_some_and(concrete_type)
        || matches!(
            init.map(Expression::without_parentheses),
            Some(Expression::ObjectExpression(_) | Expression::NewExpression(_))
        )
    {
        names.custom_names.insert(name.to_owned());
    }
}

pub(super) fn receiver(names: &Names, receiver: &Expression<'_>, method: &str) -> bool {
    let kind = names.value_kind(receiver, 0);
    if let Some(kind) = kind {
        return match method {
            "toFixed" | "toExponential" | "toPrecision" => kind != ValueKind::Number,
            "join" => !matches!(kind, ValueKind::Array { .. }),
            "has" => !matches!(kind, ValueKind::Builtin),
            _ => false,
        };
    }
    match receiver.without_parentheses() {
        Expression::Identifier(id) => names.custom_names.contains(id.name.as_str()),
        Expression::ObjectExpression(_) | Expression::NewExpression(_) => true,
        expression => cast_type(expression).is_some_and(concrete_type),
    }
}

fn cast_type<'a, 'b>(expression: &'a Expression<'b>) -> Option<&'a TSType<'b>> {
    match expression.without_parentheses() {
        Expression::TSAsExpression(cast) => Some(&cast.type_annotation),
        Expression::TSTypeAssertion(cast) => Some(&cast.type_annotation),
        Expression::TSSatisfiesExpression(cast) => Some(&cast.type_annotation),
        Expression::TSNonNullExpression(inner) => cast_type(&inner.expression),
        _ => None,
    }
}

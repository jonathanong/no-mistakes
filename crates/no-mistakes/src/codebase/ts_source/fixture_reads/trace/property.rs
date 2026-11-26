use oxc_ast::ast::{Expression, PropertyKey};

pub(super) fn computed(expression: &Expression<'_>) -> Option<String> {
    match expression.get_inner_expression() {
        Expression::StringLiteral(literal) => Some(literal.value.as_str()?.to_owned()),
        Expression::NumericLiteral(literal) => number(literal.value),
        _ => None,
    }
}

pub(super) fn destructured(key: &PropertyKey<'_>) -> Option<String> {
    match key {
        PropertyKey::StaticIdentifier(key) => Some(key.name.to_string()),
        PropertyKey::StringLiteral(key) => Some(key.value.as_str()?.to_owned()),
        PropertyKey::NumericLiteral(key) => number(key.value),
        _ => None,
    }
}

fn number(value: f64) -> Option<String> {
    ((0.0..=f64::from(u32::MAX)).contains(&value) && value.fract() == 0.0)
        .then(|| (value as u32).to_string())
}

pub(super) fn push(path: &mut String, key: &str) -> Option<()> {
    if matches!(
        key,
        "__proto__"
            | "constructor"
            | "prototype"
            | "__defineGetter__"
            | "__defineSetter__"
            | "__lookupGetter__"
            | "__lookupSetter__"
            | "hasOwnProperty"
            | "isPrototypeOf"
            | "propertyIsEnumerable"
            | "toLocaleString"
            | "toString"
            | "valueOf"
    ) {
        return None;
    }
    path.push('/');
    for c in key.chars() {
        match c {
            '~' => path.push_str("~0"),
            '/' => path.push_str("~1"),
            _ => path.push(c),
        }
    }
    Some(())
}

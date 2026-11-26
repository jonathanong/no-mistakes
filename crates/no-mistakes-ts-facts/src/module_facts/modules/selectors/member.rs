use super::*;

/// The export name read from `param` by `m.Name` or `m["Name"]`, or `None` for
/// optional, computed-by-variable or non-parameter reads.
pub(super) fn read(expression: &Expression<'_>, param: &str) -> Option<String> {
    let (object, name, optional) = match expression.get_inner_expression() {
        Expression::StaticMemberExpression(read) => {
            (&read.object, read.property.name.to_string(), read.optional)
        }
        Expression::ComputedMemberExpression(read) => {
            let Expression::StringLiteral(key) = read.expression.get_inner_expression() else {
                return None;
            };
            (
                &read.object,
                crate::ast::js_literal(key).into_owned(),
                read.optional,
            )
        }
        _ => return None,
    };
    let Expression::Identifier(object) = object.get_inner_expression() else {
        return None;
    };
    (!optional && object.name == param).then_some(name)
}

/// The export name proven by a `.then` callback body: `m.Name`, or the sole
/// `default` property of an object literal (`({ default: m.Name })`, the
/// `next/dynamic` idiom). Any other key, spread or computed key may read other
/// names, so it stays unproven.
pub(super) fn body(expression: &Expression<'_>, param: &str) -> Option<String> {
    let Expression::ObjectExpression(object) = expression.get_inner_expression() else {
        return read(expression, param);
    };
    let [ObjectPropertyKind::ObjectProperty(property)] = object.properties.as_slice() else {
        return None;
    };
    if property.computed || property.method || property.kind != PropertyKind::Init {
        return None;
    }
    (crate::ast::property_key_name(&property.key)? == "default")
        .then(|| read(&property.value, param))?
}

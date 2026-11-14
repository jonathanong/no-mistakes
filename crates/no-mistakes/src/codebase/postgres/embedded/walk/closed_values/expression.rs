use super::*;
use crate::codebase::ts_source::static_property_key_name;
fn node(expr: &Expression<'_>, depth: u8) -> Option<Node> {
    if depth == 0 {
        return None;
    }
    match unwrap_ts_wrappers(expr) {
        Expression::StringLiteral(value) => {
            Some(Node::Text(crate::ast::js_literal(value).into_owned()))
        }
        Expression::ObjectExpression(object) => {
            let mut fields = BTreeMap::new();
            for property in &object.properties {
                let ObjectPropertyKind::ObjectProperty(property) = property else {
                    return None;
                };
                if property.computed || property.method || property.kind != PropertyKind::Init {
                    return None;
                }
                let key = static_property_key_name(&property.key)?;
                if key == "__proto__" {
                    return None;
                }
                let value = node(&property.value, depth - 1)?;
                if matches!(value, Node::Array(_)) {
                    return None;
                }
                fields.insert(key.to_string(), value);
            }
            Some(Node::Object(fields))
        }
        Expression::ArrayExpression(array) => Some(Node::Array(
            array
                .elements
                .iter()
                .map(|item| node(item.as_expression()?, depth - 1))
                .collect::<Option<_>>()?,
        )),
        _ => None,
    }
}
pub(super) fn expression(
    visitor: &ScopeVisitor<'_>,
    expr: &Expression<'_>,
    depth: u8,
) -> Option<Vec<Arm>> {
    if depth == 0 {
        return None;
    }
    let expr = unwrap_ts_wrappers(expr);
    if let Some(node) = node(expr, depth) {
        return Some(vec![Arm {
            node,
            choices: vec![],
            root: None,
            provenance: None,
        }]);
    }
    match expr {
        Expression::Identifier(id) => lookup(visitor, id.name.as_str()),
        Expression::StaticMemberExpression(member) if !member.optional => select(
            expression(visitor, &member.object, depth - 1)?,
            vec![Arm {
                node: Node::Text(member.property.name.to_string()),
                choices: vec![],
                root: None,
                provenance: None,
            }],
        ),
        Expression::ComputedMemberExpression(member) if !member.optional => select(
            expression(visitor, &member.object, depth - 1)?,
            expression(visitor, &member.expression, depth - 1)?,
        ),
        Expression::NewExpression(created)
            if visitor.set_intrinsic_safe
                && visitor.lookup("Set").is_none()
                && created.arguments.len() == 1
                && matches!(unwrap_ts_wrappers(&created.callee),Expression::Identifier(id) if id.name=="Set") =>
        {
            expression(visitor, created.arguments[0].as_expression()?, depth - 1)
        }
        Expression::CallExpression(call) => assertions::recover(visitor, call, depth),
        _ => None,
    }
}
pub(super) fn select(objects: Vec<Arm>, keys: Vec<Arm>) -> Option<Vec<Arm>> {
    let mut result = Vec::new();
    for object in objects {
        for key in &keys {
            let Node::Object(fields) = &object.node else {
                return None;
            };
            let Node::Text(key_name) = &key.node else {
                return None;
            };
            if !compatible(&object.choices, &key.choices) {
                continue;
            }
            let mut arm = object.clone();
            arm.node = fields.get(key_name)?.clone();
            arm.choices.extend(&key.choices);
            result.push(arm);
        }
    }
    (result.len() <= MAX_EMBEDDED_SQL_VARIANTS && !result.is_empty()).then_some(result)
}
fn compatible(left: &[(u64, u32)], right: &[(u64, u32)]) -> bool {
    !left.iter().any(|(id, arm)| {
        right
            .iter()
            .any(|(other, branch)| id == other && arm != branch)
    })
}

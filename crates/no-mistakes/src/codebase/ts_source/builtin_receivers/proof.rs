use super::names::{Names, PrimitiveKind, ValueKind};
use super::tables::GLOBAL_OBJECTS;
use super::values::{self, constructor_kind};
use oxc_ast::ast::*;

impl Names {
    fn named(&self, name: &str) -> Option<ValueKind> {
        match self.map.get(name) {
            None if GLOBAL_OBJECTS.contains(&name) => Some(ValueKind::Namespace),
            None => None,
            Some(all) => agreed(all),
        }
    }

    pub(super) fn value_kind(&self, expr: &Expression<'_>, depth: u8) -> Option<ValueKind> {
        if depth > 8 {
            return None;
        }
        match expr.without_parentheses() {
            Expression::StringLiteral(_) | Expression::TemplateLiteral(_) => {
                Some(ValueKind::String)
            }
            Expression::NumericLiteral(_) => Some(ValueKind::Number),
            Expression::BigIntLiteral(_) => Some(ValueKind::BigInt),
            Expression::BooleanLiteral(_) => Some(ValueKind::Boolean),
            Expression::RegExpLiteral(_) => Some(ValueKind::Builtin),
            Expression::ArrayExpression(array) => Some(ValueKind::Array {
                element: None,
                empty: array
                    .elements
                    .iter()
                    .all(|item| matches!(item, ArrayExpressionElement::Elision(_))),
                length: super::array_escapes::literal_length(array),
            }),
            Expression::Identifier(id) => self.named(id.name.as_str()),
            Expression::StaticMemberExpression(member) => {
                let name = identifier_name(&member.object)?;
                let values = self
                    .properties
                    .get(name)?
                    .get(member.property.name.as_str())?;
                (self.map.get(name)?.len() == values.len())
                    .then(|| agreed(values))
                    .flatten()
            }
            Expression::NewExpression(new) => {
                let Expression::Identifier(id) = &new.callee else {
                    return None;
                };
                self.constructor(&id.name).then(|| {
                    if self.storage_constructors.contains(id.name.as_str()) {
                        ValueKind::AsyncStorage
                    } else {
                        constructor_kind(id.name.as_str())
                    }
                })
            }
            Expression::TSAsExpression(cast) if cast.type_annotation.is_const_type_reference() => {
                self.value_kind(&cast.expression, depth + 1)
            }
            Expression::TSAsExpression(cast) => self.type_kind(&cast.type_annotation),
            Expression::TSNonNullExpression(inner) => self.value_kind(&inner.expression, depth + 1),
            Expression::ConditionalExpression(cond) => {
                let left = self.value_kind(&cond.consequent, depth + 1)?;
                let right = self.value_kind(&cond.alternate, depth + 1)?;
                (left == right).then_some(left)
            }
            Expression::CallExpression(call) => values::call_result(self, call, depth + 1),
            _ => None,
        }
    }

    pub(super) fn callback_positions(
        &self,
        receiver: &Expression<'_>,
        method: &str,
    ) -> Option<&'static [usize]> {
        if matches!(method, "parse" | "stringify")
            && matches!(receiver.without_parentheses(),
                Expression::Identifier(id) if id.name == "JSON" && self.global(&id.name))
        {
            return Some(&[1]);
        }
        if method == "from"
            && matches!(receiver.without_parentheses(),
                Expression::Identifier(id) if id.name == "Array" && self.global(&id.name))
        {
            return Some(&[1]);
        }
        match self.value_kind(receiver, 0)? {
            ValueKind::Array { .. } if values::ARRAY_CALLBACKS.contains(&method) => Some(&[0]),
            ValueKind::Promise if method == "then" => Some(&[0, 1]),
            ValueKind::Promise if matches!(method, "catch" | "finally") => Some(&[0]),
            ValueKind::AsyncStorage if method == "run" => Some(&[1]),
            ValueKind::AsyncStorage if method == "exit" => Some(&[0]),
            ValueKind::String if matches!(method, "replace" | "replaceAll") => Some(&[1]),
            _ => None,
        }
    }

    pub(super) fn array_element(&self, receiver: &Expression<'_>) -> Option<PrimitiveKind> {
        match self.value_kind(receiver, 0)? {
            ValueKind::Array {
                element,
                empty: false,
                ..
            } => element,
            _ => None,
        }
    }

    pub(super) fn builtin(&self, expr: &Expression<'_>, method: Option<&str>, depth: u8) -> bool {
        if matches!(method, Some(name) if values::callback_name(name))
            && self.callback_positions(expr, method.unwrap()).is_none()
        {
            return false;
        }
        if let Some(kind) = self.value_kind(expr, depth) {
            return kind != ValueKind::Namespace || self.has_builtin_object(expr);
        }
        match expr.without_parentheses() {
            Expression::ObjectExpression(object) => method.is_some_and(|method| {
                object.properties.iter().all(|property| match property {
                    ObjectPropertyKind::ObjectProperty(property) => {
                        !property.computed
                            && property.key.static_name().is_some_and(|key| key != method)
                    }
                    ObjectPropertyKind::SpreadProperty(_) => false,
                })
            }),
            _ => false,
        }
    }

    fn has_builtin_object(&self, expr: &Expression<'_>) -> bool {
        match expr.without_parentheses() {
            Expression::Identifier(id) => GLOBAL_OBJECTS.contains(&id.name.as_str()),
            _ => false,
        }
    }
}

fn agreed(values: &[Option<ValueKind>]) -> Option<ValueKind> {
    let first = values.first().copied().flatten()?;
    if values.iter().all(|value| *value == Some(first)) {
        return Some(first);
    }
    let ValueKind::Array {
        element,
        empty,
        length,
    } = first
    else {
        return None;
    };
    values
        .iter()
        .all(|value| matches!(value, Some(ValueKind::Array { .. })))
        .then(|| ValueKind::Array {
            element: values
                .iter()
                .all(|value| {
                    matches!(value,
                Some(ValueKind::Array { element: other, empty: other_empty, length: other_len })
                    if *other == element && *other_empty == empty
                        && (*other_len == Some(0)) == (length == Some(0)))
                })
                .then_some(element)
                .flatten(),
            empty: values
                .iter()
                .all(|value| matches!(value, Some(ValueKind::Array { empty: true, .. }))),
            length: match values.first().copied().flatten() {
                Some(ValueKind::Array {
                    length: Some(len), ..
                }) if values.iter().all(|value| {
                    matches!(value,
                        Some(ValueKind::Array { length: Some(other), .. }) if *other == len)
                }) =>
                {
                    Some(len)
                }
                _ => None,
            },
        })
}

fn identifier_name<'n>(expr: &'n Expression<'_>) -> Option<&'n str> {
    match expr.without_parentheses() {
        Expression::Identifier(id) => Some(id.name.as_str()),
        _ => None,
    }
}

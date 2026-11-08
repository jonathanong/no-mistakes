use super::tables::*;
use crate::fx::FxHashMap;
use oxc_ast::ast::*;

/// Per name, whether each declaration is a built-in value.
#[derive(Default)]
pub(super) struct Names {
    pub(super) map: FxHashMap<String, Vec<bool>>,
}

pub(super) fn type_builtin(ty: &TSType<'_>) -> bool {
    match ty {
        TSType::TSStringKeyword(_)
        | TSType::TSNumberKeyword(_)
        | TSType::TSBooleanKeyword(_)
        | TSType::TSBigIntKeyword(_)
        | TSType::TSSymbolKeyword(_)
        | TSType::TSUndefinedKeyword(_)
        | TSType::TSNullKeyword(_)
        | TSType::TSLiteralType(_)
        | TSType::TSArrayType(_)
        | TSType::TSTupleType(_) => true,
        TSType::TSParenthesizedType(ty) => type_builtin(&ty.type_annotation),
        TSType::TSTypeOperatorType(ty) => type_builtin(&ty.type_annotation),
        TSType::TSUnionType(ty) => ty.types.iter().all(type_builtin),
        TSType::TSTypeReference(ty) => matches!(&ty.type_name,
            TSTypeName::IdentifierReference(name) if TYPES.contains(&name.name.as_str())),
        _ => false,
    }
}

impl Names {
    pub(super) fn global(&self, name: &str) -> bool {
        !self.map.contains_key(name)
    }

    fn constructor(&self, name: &str) -> bool {
        self.map
            .get(name)
            .is_none_or(|all| all.iter().all(|builtin| *builtin))
    }

    fn named(&self, name: &str) -> bool {
        match self.map.get(name) {
            None => GLOBAL_OBJECTS.contains(&name),
            Some(all) => all.iter().all(|builtin| *builtin),
        }
    }

    /// `method` is the method called on `expr`, when the expression is a call receiver.
    pub(super) fn builtin(&self, expr: &Expression<'_>, method: Option<&str>, depth: u8) -> bool {
        if depth > 8 {
            return false;
        }
        match expr.without_parentheses() {
            Expression::StringLiteral(_)
            | Expression::NumericLiteral(_)
            | Expression::BigIntLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::RegExpLiteral(_)
            | Expression::TemplateLiteral(_)
            | Expression::ArrayExpression(_) => true,
            Expression::ObjectExpression(object) => method.is_some_and(|method| {
                object.properties.iter().all(|property| match property {
                    ObjectPropertyKind::ObjectProperty(property) => {
                        !property.computed
                            && property.key.static_name().is_some_and(|key| key != method)
                    }
                    ObjectPropertyKind::SpreadProperty(_) => false,
                })
            }),
            Expression::Identifier(id) => self.named(id.name.as_str()),
            Expression::NewExpression(new) => matches!(&new.callee,
                Expression::Identifier(id)
                    if self.constructor(&id.name) && CONSTRUCTORS.contains(&id.name.as_str())),
            Expression::TSNonNullExpression(inner) => {
                self.builtin(&inner.expression, method, depth + 1)
            }
            Expression::TSAsExpression(cast) => type_builtin(&cast.type_annotation),
            Expression::ConditionalExpression(cond) => {
                self.builtin(&cond.consequent, method, depth + 1)
                    && self.builtin(&cond.alternate, method, depth + 1)
            }
            Expression::CallExpression(call) => self.builtin_call(call, depth),
            _ => false,
        }
    }

    fn builtin_call(&self, call: &CallExpression<'_>, depth: u8) -> bool {
        match call.callee.without_parentheses() {
            Expression::Identifier(id) => {
                self.global(&id.name) && matches!(id.name.as_str(), "String" | "Number" | "Boolean")
            }
            Expression::StaticMemberExpression(member) => {
                let name = member.property.name.as_str();
                match &member.object {
                    Expression::Identifier(id) if self.global(&id.name) => {
                        FACTORIES.contains(&(id.name.as_str(), name))
                    }
                    object => {
                        PRESERVING.contains(&name) && self.builtin(object, Some(name), depth + 1)
                    }
                }
            }
            _ => false,
        }
    }
}

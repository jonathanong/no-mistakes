use super::super::names::{Names, PrimitiveKind, ValueKind};
use super::super::tables::TYPES;
use oxc_ast::ast::*;

pub(in super::super) fn value_kind_from_type(ty: &TSType<'_>) -> Option<ValueKind> {
    match ty {
        TSType::TSStringKeyword(_) => Some(ValueKind::String),
        TSType::TSNumberKeyword(_) => Some(ValueKind::Number),
        TSType::TSBooleanKeyword(_) => Some(ValueKind::Boolean),
        TSType::TSBigIntKeyword(_) => Some(ValueKind::BigInt),
        TSType::TSLiteralType(literal) => Some(match literal.literal {
            TSLiteral::StringLiteral(_) => ValueKind::String,
            TSLiteral::NumericLiteral(_) => ValueKind::Number,
            TSLiteral::BooleanLiteral(_) => ValueKind::Boolean,
            _ => return None,
        }),
        TSType::TSArrayType(array) => Some(ValueKind::Array {
            element: value_kind_from_type(&array.element_type).and_then(PrimitiveKind::from_value),
            empty: false,
            length: None,
        }),
        TSType::TSTupleType(_) => Some(ValueKind::Array {
            element: None,
            empty: false,
            length: None,
        }),
        TSType::TSTypeReference(reference) => reference_kind(reference),
        TSType::TSParenthesizedType(inner) => value_kind_from_type(&inner.type_annotation),
        TSType::TSTypeOperatorType(operator) => value_kind_from_type(&operator.type_annotation),
        TSType::TSUnionType(union) => union_kind(union),
        TSType::TSNullKeyword(_) | TSType::TSUndefinedKeyword(_) | TSType::TSSymbolKeyword(_) => {
            Some(ValueKind::Builtin)
        }
        _ => None,
    }
}

fn union_kind(union: &TSUnionType<'_>) -> Option<ValueKind> {
    let kinds = union
        .types
        .iter()
        .map(value_kind_from_type)
        .collect::<Option<Vec<_>>>()?;
    let first = *kinds.first()?;
    if kinds.iter().all(|kind| *kind == first) {
        return Some(first);
    }
    if kinds
        .iter()
        .all(|kind| matches!(kind, ValueKind::Array { .. }))
    {
        return Some(ValueKind::Array {
            element: None,
            empty: false,
            length: None,
        });
    }
    kinds
        .iter()
        .all(|kind| type_is_builtin(*kind))
        .then_some(ValueKind::Builtin)
}

fn reference_kind(reference: &TSTypeReference<'_>) -> Option<ValueKind> {
    let TSTypeName::IdentifierReference(name) = &reference.type_name else {
        return None;
    };
    match name.name.as_str() {
        "Array" | "ReadonlyArray" => Some(ValueKind::Array {
            element: reference
                .type_arguments
                .as_ref()?
                .params
                .first()
                .and_then(value_kind_from_type)
                .and_then(PrimitiveKind::from_value),
            empty: false,
            length: None,
        }),
        "Promise" | "PromiseLike" => Some(ValueKind::Promise),
        "AsyncLocalStorage" => Some(ValueKind::Builtin),
        value if TYPES.contains(&value) => Some(ValueKind::Builtin),
        _ => None,
    }
}

fn type_is_builtin(kind: ValueKind) -> bool {
    matches!(
        kind,
        ValueKind::String
            | ValueKind::Number
            | ValueKind::Boolean
            | ValueKind::BigInt
            | ValueKind::Array { .. }
            | ValueKind::Promise
            | ValueKind::AsyncStorage
            | ValueKind::Builtin
    )
}

pub(in super::super) fn property_types(
    names: &Names,
    ty: &TSType<'_>,
) -> Vec<(String, Option<ValueKind>)> {
    let TSType::TSTypeLiteral(literal) = ty else {
        return Vec::new();
    };
    literal
        .members
        .iter()
        .filter_map(|member| {
            let TSSignature::TSPropertySignature(property) = member else {
                return None;
            };
            if property.computed {
                return None;
            }
            let key = property.key.static_name()?;
            let value = property
                .type_annotation
                .as_ref()
                .and_then(|annotation| names.type_kind(&annotation.type_annotation));
            Some((key.into_cow_str()?.into_owned(), value))
        })
        .collect()
}

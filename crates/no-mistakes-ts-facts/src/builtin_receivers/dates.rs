use super::names::Names;
use oxc_ast::ast::*;

pub(super) fn note(
    names: &mut Names,
    prior: &Names,
    name: &str,
    ty: Option<&TSType<'_>>,
    init: Option<&Expression<'_>>,
) {
    names.note_date(name, is_date(prior, ty, init));
}

pub(super) fn receiver(names: &Names, expr: &Expression<'_>) -> bool {
    match expr.without_parentheses() {
        Expression::NewExpression(new) => new_date(names, new),
        Expression::Identifier(id) => names.date_name(id.name.as_str()),
        Expression::TSNonNullExpression(inner) => receiver(names, &inner.expression),
        Expression::TSAsExpression(cast) => date_type(names, &cast.type_annotation),
        Expression::TSTypeAssertion(cast) => date_type(names, &cast.type_annotation),
        _ => false,
    }
}

fn is_date(prior: &Names, ty: Option<&TSType<'_>>, init: Option<&Expression<'_>>) -> bool {
    ty.is_some_and(|ty| date_type(prior, ty)) || init.is_some_and(|init| init_date(prior, init))
}

fn init_date(names: &Names, expr: &Expression<'_>) -> bool {
    match expr.without_parentheses() {
        Expression::NewExpression(new) => new_date(names, new),
        Expression::TSAsExpression(cast) => date_type(names, &cast.type_annotation),
        Expression::TSTypeAssertion(cast) => date_type(names, &cast.type_annotation),
        Expression::TSNonNullExpression(inner) => init_date(names, &inner.expression),
        _ => false,
    }
}

fn new_date(names: &Names, new: &NewExpression<'_>) -> bool {
    matches!(&new.callee, Expression::Identifier(id)
        if id.name == "Date" && names.constructor(id.name.as_str()))
}

fn date_type(names: &Names, ty: &TSType<'_>) -> bool {
    match ty {
        TSType::TSTypeReference(reference) => {
            matches!(&reference.type_name, TSTypeName::IdentifierReference(name)
                if name.name == "Date" && names.constructor(name.name.as_str()))
        }
        TSType::TSParenthesizedType(inner) => date_type(names, &inner.type_annotation),
        _ => false,
    }
}

use super::*;

pub(in crate::codebase::postgres::embedded::walk) fn recover_relation(
    visitor: &ScopeVisitor<'_>,
    expr: &Expression<'_>,
) -> Option<Vec<Recovered>> {
    if let Some(values) = super::recover(visitor, expr) {
        return Some(values);
    }
    let Expression::Identifier(id) = unwrap_ts_wrappers(expr) else {
        return None;
    };
    let i = visitor
        .scopes
        .iter()
        .rposition(|scope| scope.contains_key(id.name.as_str()))?;
    let arms = visitor
        .literal_objects
        .closed
        .relation_frames
        .get(i)?
        .get(id.name.as_str())?
        .clone();
    super::materialize_arms(visitor, expr, arms)
}

pub(super) fn indexed_array_strings(
    visitor: &ScopeVisitor<'_>,
    ty: &TSType<'_>,
) -> Option<Vec<String>> {
    let ty = match ty {
        TSType::TSParenthesizedType(inner) => &inner.type_annotation,
        _ => ty,
    };
    let TSType::TSTypeQuery(query) = ty else {
        return None;
    };
    let TSTypeQueryExprName::IdentifierReference(name) = &query.expr_name else {
        return None;
    };
    let i = visitor
        .scopes
        .iter()
        .rposition(|scope| scope.contains_key(name.name.as_str()))?;
    visitor
        .literal_objects
        .closed
        .const_literal_arrays
        .get(i)?
        .get(name.name.as_str())
        .filter(|values| !values.is_empty())
        .cloned()
}

pub(super) fn note_const_relation_and_array(
    visitor: &mut ScopeVisitor<'_>,
    type_annotation: Option<&TSTypeAnnotation<'_>>,
    init: &Expression<'_>,
    scope: usize,
    name: &str,
) {
    if let (Expression::Identifier(id), _) = unwrap_const_preserving(init) {
        let id_name = id.name.as_str();
        if let Some(i) = visitor
            .scopes
            .iter()
            .rposition(|frame| frame.contains_key(id_name))
        {
            visitor.literal_objects.closed.copy_const_alias(
                i,
                scope,
                id_name,
                name,
                type_annotation.is_none(),
            );
        }
    }
    if let Some(values) = literal_array_values(type_annotation, init) {
        if let Some(frame) = visitor
            .literal_objects
            .closed
            .const_literal_arrays
            .get_mut(scope)
        {
            frame.insert(name.to_string(), values);
        }
    }
}

fn literal_array_values(
    type_annotation: Option<&TSTypeAnnotation<'_>>,
    init: &Expression<'_>,
) -> Option<Vec<String>> {
    if let Some(annotation) = type_annotation {
        return tuple_string_literals(&annotation.type_annotation);
    }
    as_const_array_literals(init)
}

fn tuple_string_literals(mut ty: &TSType<'_>) -> Option<Vec<String>> {
    loop {
        match ty {
            TSType::TSParenthesizedType(inner) => ty = &inner.type_annotation,
            TSType::TSTypeOperatorType(op) if op.operator == TSTypeOperatorOperator::Readonly => {
                ty = &op.type_annotation;
            }
            _ => break,
        }
    }
    let TSType::TSTupleType(tuple) = ty else {
        return None;
    };
    let mut values = Vec::new();
    for element in &tuple.element_types {
        let value = tuple_element_string(element)?;
        if !values.contains(&value) {
            values.push(value);
            if values.len() > MAX_EMBEDDED_SQL_VARIANTS {
                return None;
            }
        }
    }
    (!values.is_empty()).then_some(values)
}

fn tuple_element_string(element: &TSTupleElement<'_>) -> Option<String> {
    let item = match element {
        TSTupleElement::TSNamedTupleMember(named) if named.optional => return None,
        TSTupleElement::TSNamedTupleMember(named) => &named.element_type,
        other => other,
    };
    let TSType::TSLiteralType(lit) = item.as_ts_type()? else {
        return None;
    };
    match &lit.literal {
        TSLiteral::StringLiteral(text) => Some(crate::ast::js_literal(text).into_owned()),
        _ => None,
    }
}

fn is_as_const(ty: &TSType<'_>) -> bool {
    matches!(
        ty,
        TSType::TSTypeReference(reference)
            if reference.type_arguments.is_none()
                && matches!(&reference.type_name, TSTypeName::IdentifierReference(id) if id.name == "const")
    )
}

fn unwrap_const_preserving<'a>(mut expr: &'a Expression<'a>) -> (&'a Expression<'a>, bool) {
    let mut saw_as_const = false;
    loop {
        match expr {
            Expression::ParenthesizedExpression(inner) => expr = &inner.expression,
            Expression::TSSatisfiesExpression(inner) => expr = &inner.expression,
            Expression::TSAsExpression(as_expr) if is_as_const(&as_expr.type_annotation) => {
                saw_as_const = true;
                expr = &as_expr.expression;
            }
            _ => return (expr, saw_as_const),
        }
    }
}

fn as_const_array_literals(expr: &Expression<'_>) -> Option<Vec<String>> {
    let (Expression::ArrayExpression(array), true) = unwrap_const_preserving(expr) else {
        return None;
    };
    let mut values = Vec::new();
    for item in &array.elements {
        let (Expression::StringLiteral(text), _) = unwrap_const_preserving(item.as_expression()?)
        else {
            return None;
        };
        let value = crate::ast::js_literal(text).into_owned();
        if !values.contains(&value) {
            values.push(value);
            if values.len() > MAX_EMBEDDED_SQL_VARIANTS {
                return None;
            }
        }
    }
    (!values.is_empty()).then_some(values)
}

use super::*;

pub(super) fn selected(
    import: NodeId,
    array_id: NodeId,
    array: &ArrayExpression<'_>,
    semantic: &Semantic<'_>,
) -> Option<Vec<String>> {
    if array
        .elements
        .iter()
        .any(|entry| matches!(entry, ArrayExpressionElement::SpreadElement(_)))
    {
        return None;
    }
    let index = array.elements.iter().position(|entry| {
        entry
            .span()
            .contains_inclusive(semantic.nodes().kind(import).span())
    })?;
    let (call_id, AstKind::CallExpression(call)) = parent(array_id, semantic)? else {
        return None;
    };
    let Expression::StaticMemberExpression(member) = call.callee.get_inner_expression() else {
        return None;
    };
    let Expression::Identifier(object) = member.object.get_inner_expression() else {
        return None;
    };
    if call.optional
        || member.optional
        || member.property.name != "all"
        || object.name != "Promise"
        || semantic
            .scoping()
            .get_reference(object.reference_id.get()?)
            .symbol_id()
            .is_some()
        || call.arguments.len() != 1
        || call.arguments[0].span() != array.span
    {
        return None;
    }
    let (await_id, AstKind::AwaitExpression(_)) = parent(call_id, semantic)? else {
        return None;
    };
    let (_, AstKind::VariableDeclarator(declarator)) = parent(await_id, semantic)? else {
        return None;
    };
    let BindingPattern::ArrayPattern(pattern) = &declarator.id else {
        return None;
    };
    if pattern.rest.is_some() {
        return None;
    }
    let BindingPattern::ObjectPattern(object) = pattern.elements.get(index)?.as_ref()? else {
        return None;
    };
    object_names(object)
}

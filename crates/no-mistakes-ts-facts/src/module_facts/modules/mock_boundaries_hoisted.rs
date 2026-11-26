use super::bindings::mock_binding;
use super::*;

pub(super) fn proven(expression: &Expression<'_>, semantic: &Semantic<'_>) -> bool {
    let Expression::Identifier(identifier) = expression.get_inner_expression() else {
        return false;
    };
    let Some(symbol) = identifier
        .reference_id
        .get()
        .and_then(|reference| semantic.scoping().get_reference(reference).symbol_id())
    else {
        return false;
    };
    if !semantic.scoping().symbol_flags(symbol).is_const_variable()
        || semantic
            .scoping()
            .get_resolved_references(symbol)
            .any(|reference| reference.is_write())
    {
        return false;
    }
    let declaration = semantic.scoping().symbol_declaration(symbol);
    let AstKind::VariableDeclarator(declarator) = semantic.nodes().kind(declaration) else {
        return false;
    };
    let top_level = matches!(
        semantic.nodes().parent_kind(declaration),
        AstKind::VariableDeclaration(_)
    ) && matches!(
        semantic
            .nodes()
            .parent_kind(semantic.nodes().parent_id(declaration)),
        AstKind::Program(_)
    );
    top_level
        && declarator.id.get_binding_identifier().is_some()
        && declarator
            .init
            .as_ref()
            .is_some_and(|init| initializer(init, semantic))
}

fn initializer(init: &Expression<'_>, semantic: &Semantic<'_>) -> bool {
    let Expression::CallExpression(call) = init.get_inner_expression() else {
        return false;
    };
    let Some(member) = call.callee.get_inner_expression().as_member_expression() else {
        return false;
    };
    let Expression::Identifier(object) = member.object().get_inner_expression() else {
        return false;
    };
    if call.optional
        || member.optional()
        || member.static_property_name().and_then(|name| name.as_str()) != Some("hoisted")
        || !mock_binding(object, semantic)
        || call.arguments.len() != 1
    {
        return false;
    }
    let Some(Argument::ArrowFunctionExpression(function)) = call.arguments.first() else {
        return false;
    };
    let body = function.body.as_expression().or_else(|| {
        match function.body.as_function_body()?.statements.as_slice() {
            [Statement::ReturnStatement(statement)] => statement.argument.as_ref(),
            _ => None,
        }
    });
    function.params.items.is_empty()
        && function.params.rest.is_none()
        && !function.r#async
        && matches!(
            body.map(Expression::get_inner_expression),
            Some(Expression::ObjectExpression(object)) if inert_object(object, semantic)
        )
}

fn inert_object(object: &ObjectExpression<'_>, semantic: &Semantic<'_>) -> bool {
    object.properties.iter().all(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return false;
        };
        property.kind == PropertyKind::Init
            && !property.method
            && !property.computed
            && crate::ast::property_key_name(&property.key).is_some_and(|key| key != "__proto__")
            && inert_value(&property.value, semantic)
    })
}

fn inert_value(value: &Expression<'_>, semantic: &Semantic<'_>) -> bool {
    match value.get_inner_expression() {
        Expression::ObjectExpression(object) => inert_object(object, semantic),
        Expression::TemplateLiteral(template) => template.expressions.is_empty(),
        Expression::CallExpression(call) => {
            let Some(member) = call.callee.get_inner_expression().as_member_expression() else {
                return false;
            };
            matches!(member.object().get_inner_expression(), Expression::Identifier(object) if mock_binding(object, semantic))
                && member.static_property_name().and_then(|name| name.as_str()) == Some("fn")
                && !call.optional
                && !member.optional()
                && call.arguments.iter().all(|argument| {
                    argument
                        .as_expression()
                        .is_some_and(|argument| inert_value(argument, semantic))
                })
        }
        other => other.is_literal(),
    }
}

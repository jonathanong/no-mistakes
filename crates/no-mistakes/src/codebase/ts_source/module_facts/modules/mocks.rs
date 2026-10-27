use super::*;

/// Vitest's module-promise overload erases its import expression. Require the
/// imported utility identity and default transform spellings; arbitrary aliases,
/// local objects and shadowed names remain ordinary runtime imports.
pub(super) fn typed_mock_import(
    import: &ImportExpression<'_>,
    node: oxc_semantic::NodeId,
    semantic: &Semantic<'_>,
) -> bool {
    let Some(AstKind::CallExpression(call)) = loads::unwrapped_parent(node, semantic) else {
        return false;
    };
    if call.optional
        || call.arguments.len() != 2
        || call
            .arguments
            .first()
            .is_none_or(|arg| arg.span() != import.span)
    {
        return false;
    }
    let Expression::StaticMemberExpression(member) = call.callee.get_inner_expression() else {
        return false;
    };
    if member.optional || !matches!(member.property.name.as_str(), "mock" | "doMock") {
        return false;
    }
    let Expression::Identifier(object) = member.object.get_inner_expression() else {
        return false;
    };
    if !matches!(object.name.as_str(), "vi" | "vitest") {
        return false;
    }
    let Some(symbol) = object
        .reference_id
        .get()
        .and_then(|reference| semantic.scoping().get_reference(reference).symbol_id())
    else {
        return false;
    };
    let declaration = semantic.scoping().symbol_declaration(symbol);
    let AstKind::ImportSpecifier(binding) = semantic.nodes().kind(declaration) else {
        return false;
    };
    if !matches!(name(&binding.imported).as_str(), "vi" | "vitest") || binding.import_kind.is_type()
    {
        return false;
    }
    matches!(semantic.nodes().parent_kind(declaration), AstKind::ImportDeclaration(import)
        if !import.import_kind.is_type() && crate::ast::js_literal(&import.source) == "vitest")
        && call.arguments.get(1).is_some_and(inert_factory)
}

// A factory can implicitly load the original module through an argument or an
// opaque call. Retain uncertainty rather than assuming every mock is isolated.
fn inert_factory(argument: &Argument<'_>) -> bool {
    use oxc_ast_visit::Visit;
    let (params, replacement) = match argument {
        Argument::ArrowFunctionExpression(function) => (
            &function.params,
            function
                .body
                .as_expression()
                .or_else(|| function.body.as_function_body().and_then(sole_return)),
        ),
        Argument::FunctionExpression(function) if !function.generator => (
            &function.params,
            function.body.as_deref().and_then(sole_return),
        ),
        _ => return false,
    };
    if !params.items.is_empty() || params.rest.is_some() {
        return false;
    }
    let Some(Expression::ObjectExpression(object)) =
        replacement.map(Expression::get_inner_expression)
    else {
        return false;
    };
    if !literal_object(object) {
        return false;
    }
    let mut execution = FactoryExecution(false);
    execution.visit_argument(argument);
    !execution.0
}

struct FactoryExecution(bool);

impl<'a> oxc_ast_visit::Visit<'a> for FactoryExecution {
    fn visit_call_expression(&mut self, _: &CallExpression<'a>) {
        self.0 = true;
    }
    fn visit_new_expression(&mut self, _: &NewExpression<'a>) {
        self.0 = true;
    }
    fn visit_tagged_template_expression(&mut self, _: &TaggedTemplateExpression<'a>) {
        self.0 = true;
    }
    fn visit_import_expression(&mut self, _: &ImportExpression<'a>) {
        self.0 = true;
    }
}

fn sole_return<'a>(body: &'a FunctionBody<'a>) -> Option<&'a Expression<'a>> {
    let [Statement::ReturnStatement(statement)] = body.statements.as_slice() else {
        return None;
    };
    statement.argument.as_ref()
}

fn literal_object(object: &ObjectExpression<'_>) -> bool {
    object.properties.iter().all(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return false;
        };
        property.kind == PropertyKind::Init
            && !property.method
            && !property.computed
            && crate::ast::property_key_name(&property.key).is_some_and(|key| key != "__proto__")
            && literal_value(&property.value)
    })
}

fn literal_value(value: &Expression<'_>) -> bool {
    match value.get_inner_expression() {
        Expression::StringLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::BigIntLiteral(_)
        | Expression::BooleanLiteral(_)
        | Expression::NullLiteral(_)
        | Expression::ArrowFunctionExpression(_)
        | Expression::FunctionExpression(_) => true,
        Expression::ObjectExpression(object) => literal_object(object),
        Expression::ArrayExpression(array) => array
            .elements
            .iter()
            .all(|element| element.as_expression().is_some_and(literal_value)),
        _ => false,
    }
}

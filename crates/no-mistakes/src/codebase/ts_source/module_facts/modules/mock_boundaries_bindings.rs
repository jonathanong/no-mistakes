use super::*;

pub(super) fn mock_method<'a>(
    call: &CallExpression<'a>,
    semantic: &Semantic<'_>,
) -> Option<(&'a str, bool)> {
    let member = call.callee.get_inner_expression().as_member_expression()?;
    let method = member.static_property_name()?.as_str()?;
    if !matches!(method, "mock" | "doMock" | "unmock" | "doUnmock") {
        return None;
    }
    let Expression::Identifier(object) = member.object().get_inner_expression() else {
        return None;
    };
    if !mock_binding(object, semantic) {
        return None;
    }
    Some((method, member.optional()))
}

pub(super) fn mock_binding(object: &IdentifierReference<'_>, semantic: &Semantic<'_>) -> bool {
    let Some(reference) = object.reference_id.get() else {
        return false;
    };
    let Some(symbol) = semantic.scoping().get_reference(reference).symbol_id() else {
        return matches!(object.name.as_str(), "vi" | "jest");
    };
    let declaration = semantic.scoping().symbol_declaration(symbol);
    let AstKind::ImportSpecifier(binding) = semantic.nodes().kind(declaration) else {
        return false;
    };
    if binding.import_kind.is_type() {
        return false;
    }
    let AstKind::ImportDeclaration(import) = semantic.nodes().parent_kind(declaration) else {
        return false;
    };
    if import.import_kind.is_type() {
        return false;
    }
    let imported = name(&binding.imported);
    let source = crate::ast::js_literal(&import.source);
    matches!(
        (source.as_ref(), imported.as_str()),
        ("vitest", "vi" | "vitest") | ("@jest/globals", "jest")
    )
}

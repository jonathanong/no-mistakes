use oxc_ast::{ast::*, AstKind};
use oxc_semantic::{NodeId, Semantic};
use oxc_syntax::symbol::SymbolId;

pub(super) fn assigned_capture(
    semantic: &Semantic<'_>,
    function: NodeId,
    call: &CallExpression<'_>,
) -> Option<String> {
    let assignment = assignment_parent(semantic, function)?;
    if assignment.operator != AssignmentOperator::Assign {
        return None;
    }
    let AssignmentTarget::StaticMemberExpression(target) = &assignment.left else {
        return None;
    };
    let Expression::Identifier(callee) = call.callee.get_inner_expression() else {
        return None;
    };
    let symbol = reference_symbol(semantic, callee)?;
    if !semantic.scoping().symbol_flags(symbol).is_const_variable()
        || semantic
            .scoping()
            .get_resolved_references(symbol)
            .any(|r| r.flags().is_write())
    {
        return None;
    }
    let capture = semantic
        .nodes()
        .kind(semantic.scoping().symbol_declaration(symbol))
        .as_variable_declarator()?;
    if !matches!(&capture.id, BindingPattern::BindingIdentifier(id) if id.symbol_id.get() == Some(symbol))
        || capture.span.end > assignment.span.start
    {
        return None;
    }
    let Expression::StaticMemberExpression(original) =
        capture.init.as_ref()?.get_inner_expression()
    else {
        return None;
    };
    let same_member = target.property.name == original.property.name
        && imported_object(semantic, &target.object)?
            == imported_object(semantic, &original.object)?;
    same_member.then(|| target.property.name.to_string())
}

fn reference_symbol(semantic: &Semantic<'_>, id: &IdentifierReference<'_>) -> Option<SymbolId> {
    semantic
        .scoping()
        .get_reference(id.reference_id.get()?)
        .symbol_id()
}

fn imported_object(semantic: &Semantic<'_>, object: &Expression<'_>) -> Option<SymbolId> {
    let Expression::Identifier(id) = object.get_inner_expression() else {
        return None;
    };
    let symbol = reference_symbol(semantic, id)?;
    let declaration = semantic.scoping().symbol_declaration(symbol);
    let projection = match semantic.nodes().kind(declaration) {
        AstKind::ImportDefaultSpecifier(_) | AstKind::ImportNamespaceSpecifier(_) => "",
        AstKind::ImportSpecifier(specifier)
            if !specifier.import_kind.is_type()
                && matches!(&specifier.imported, ModuleExportName::IdentifierName(name) if name.name == "promises") =>
        {
            "promises"
        }
        _ => return None,
    };
    let import = semantic
        .nodes()
        .parent_kind(declaration)
        .as_import_declaration()?;
    let source = crate::ast::js_literal(&import.source);
    let module = source.strip_prefix("node:").unwrap_or(&source);
    (semantic.scoping().symbol_redeclarations(symbol).is_empty()
        && !import.import_kind.is_type()
        && (projection.is_empty() || module == "fs")
        && matches!(
            source.strip_prefix("node:").unwrap_or(&source),
            "fs" | "fs/promises" | "fs-extra"
        )
        && !semantic
            .scoping()
            .get_resolved_references(symbol)
            .any(|r| r.flags().is_write()))
    .then_some(symbol)
}

pub(crate) fn assignment_parent<'a>(
    semantic: &Semantic<'a>,
    function: NodeId,
) -> Option<&'a AssignmentExpression<'a>> {
    let mut current = semantic.nodes().parent_id(function);
    for _ in 0..16 {
        match semantic.nodes().kind(current) {
            AstKind::AssignmentExpression(assignment) => return Some(assignment),
            AstKind::ParenthesizedExpression(_)
            | AstKind::TSAsExpression(_)
            | AstKind::TSSatisfiesExpression(_)
            | AstKind::TSNonNullExpression(_)
            | AstKind::TSTypeAssertion(_) => current = semantic.nodes().parent_id(current),
            _ => return None,
        }
    }
    None
}

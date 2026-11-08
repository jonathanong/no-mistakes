use super::*;

pub(super) fn selected(
    arrow: NodeId,
    function: &ArrowFunctionExpression<'_>,
    semantic: &Semantic<'_>,
    calls: &mut Vec<String>,
) -> Option<Vec<String>> {
    if !function.is_expression()
        || function.r#async
        || !function.params.items.is_empty()
        || function.params.rest.is_some()
    {
        return None;
    }
    let (property_id, AstKind::ObjectProperty(property)) = parent(arrow, semantic)? else {
        return None;
    };
    let key = crate::ast::property_key_name(&property.key)?;
    let (symbol, keys) = local_registry(property_id, semantic)?;
    let members =
        super::super::imports_namespace::runtime_members(semantic, symbol, &mut Vec::new())?;
    if members.iter().any(|name| !keys.contains(name.as_str())) {
        return None;
    }
    let mut names = Vec::new();
    for reference in semantic.scoping().get_resolved_references(symbol) {
        let (member_id, AstKind::StaticMemberExpression(member)) =
            parent(reference.node_id(), semantic)?
        else {
            return None;
        };
        if member.optional {
            return None;
        }
        let (call_id, AstKind::CallExpression(call)) = parent(member_id, semantic)? else {
            return None;
        };
        if call.optional || call.callee.span() != member.span || !call.arguments.is_empty() {
            return None;
        }
        if member.property.name.as_str() != key.as_ref() {
            continue;
        }
        let (await_id, AstKind::AwaitExpression(_)) = parent(call_id, semantic)? else {
            return None;
        };
        names.extend(awaited(await_id, semantic, calls)?);
    }
    (!names.is_empty()).then_some(names)
}

fn local_registry(
    property_id: NodeId,
    semantic: &Semantic<'_>,
) -> Option<(oxc_syntax::symbol::SymbolId, crate::fx::FxHashSet<String>)> {
    let (object_id, kind) = parent(property_id, semantic)?;
    let object = kind.as_object_expression()?;
    let mut keys = crate::fx::FxHashSet::default();
    for entry in &object.properties {
        let ObjectPropertyKind::ObjectProperty(entry) = entry else {
            return None;
        };
        if entry.computed
            || entry.method
            || entry.kind != PropertyKind::Init
            || !matches!(
                entry.value.get_inner_expression(),
                Expression::ArrowFunctionExpression(_)
            )
            || !keys.insert(crate::ast::property_key_name(&entry.key)?.into_owned())
        {
            return None;
        }
    }
    let (declarator_id, AstKind::VariableDeclarator(declarator)) = parent(object_id, semantic)?
    else {
        return None;
    };
    let BindingPattern::BindingIdentifier(binding) = &declarator.id else {
        return None;
    };
    let (declaration_id, kind) = parent(declarator_id, semantic)?;
    let declaration = kind.as_variable_declaration()?;
    if declaration.kind != VariableDeclarationKind::Const
        || matches!(
            parent(declaration_id, semantic)?.1,
            AstKind::ExportNamedDeclaration(_) | AstKind::ExportDeclaration(_)
        )
    {
        return None;
    }
    Some((binding.symbol_id(), keys))
}

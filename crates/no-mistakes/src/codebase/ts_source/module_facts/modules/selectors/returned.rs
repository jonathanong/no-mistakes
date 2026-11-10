use super::*;

/// `function load() { return import("./x") }` then `const { Name } = await load()`.
///
/// One same-file local function, whose only return is this literal import, and
/// whose every reference is an awaited call destructured by static keys.
/// Documented in `docs/rules/production-exports-only-used-by-tests.md`.
pub(super) fn selected(return_id: NodeId, semantic: &Semantic<'_>) -> Option<Vec<String>> {
    let (function, symbol) = local_symbol(return_id, semantic)?;
    if !only_this_return(function, semantic.nodes().kind(return_id).span(), semantic) {
        return None;
    }
    let mut names = Vec::new();
    let mut calls = 0u32;
    for reference in semantic.scoping().get_resolved_references(symbol) {
        if reference.is_write() || reference.is_type() {
            return None;
        }
        names.extend(awaited_keys(reference.node_id(), semantic)?);
        calls += 1;
    }
    if calls == 0 {
        return None;
    }
    names.sort_unstable();
    names.dedup();
    Some(names)
}

fn local_symbol(
    return_id: NodeId,
    semantic: &Semantic<'_>,
) -> Option<(NodeId, oxc_syntax::symbol::SymbolId)> {
    let function = semantic.nodes().ancestors(return_id).find(|node| {
        matches!(
            node.kind(),
            AstKind::Function(_) | AstKind::ArrowFunctionExpression(_)
        )
    })?;
    let function_id = function.id();
    if matches!(function.kind(), AstKind::Function(function) if function.generator) {
        return None;
    }
    if let Some(symbol) = const_symbol(function_id, semantic) {
        return Some((function_id, symbol));
    }
    let AstKind::Function(function) = semantic.nodes().kind(function_id) else {
        return None;
    };
    if function.r#type != FunctionType::FunctionDeclaration || function.declare {
        return None;
    }
    if exported(function_id, semantic) {
        return None;
    }
    Some((function_id, function.id.as_ref()?.symbol_id.get()?))
}

fn const_symbol(function: NodeId, semantic: &Semantic<'_>) -> Option<oxc_syntax::symbol::SymbolId> {
    let (declarator_id, kind) = parent(function, semantic)?;
    let declarator = kind.as_variable_declarator()?;
    let BindingPattern::BindingIdentifier(binding) = &declarator.id else {
        return None;
    };
    let (declaration, declaration_kind) = parent(declarator_id, semantic)?;
    let variable = declaration_kind.as_variable_declaration()?;
    if variable.kind != VariableDeclarationKind::Const || exported(declaration, semantic) {
        return None;
    }
    binding.symbol_id.get()
}

fn exported(node: NodeId, semantic: &Semantic<'_>) -> bool {
    matches!(
        parent(node, semantic).map(|(_, kind)| kind),
        Some(
            AstKind::ExportNamedDeclaration(_)
                | AstKind::ExportDefaultDeclaration(_)
                | AstKind::ExportDeclaration(_)
        )
    )
}

fn only_this_return(
    function: NodeId,
    return_span: oxc_span::Span,
    semantic: &Semantic<'_>,
) -> bool {
    let Some(body_span) = body_span(function, semantic) else {
        return false;
    };
    let mut found = false;
    for node in semantic.nodes().iter() {
        let AstKind::ReturnStatement(statement) = node.kind() else {
            continue;
        };
        if !body_span.contains_inclusive(statement.span)
            || !belongs_to(node.id(), function, semantic)
        {
            continue;
        }
        if found || statement.span != return_span {
            return false;
        }
        found = true;
    }
    found
}

fn body_span(function: NodeId, semantic: &Semantic<'_>) -> Option<oxc_span::Span> {
    let kind = semantic.nodes().kind(function);
    if let Some(function) = kind.as_function() {
        return function.body.as_ref().map(|body| body.span);
    }
    // A class static block can hold a recovered return under an expression-bodied arrow.
    Some(
        kind.as_arrow_function_expression()?
            .body
            .as_function_body()?
            .span,
    )
}

fn belongs_to(node: NodeId, function: NodeId, semantic: &Semantic<'_>) -> bool {
    semantic
        .nodes()
        .ancestors(node)
        .find(|ancestor| {
            ancestor.id() == function
                || matches!(
                    ancestor.kind(),
                    AstKind::Function(_) | AstKind::ArrowFunctionExpression(_)
                )
        })
        .is_some_and(|ancestor| ancestor.id() == function)
}

fn awaited_keys(reference: NodeId, semantic: &Semantic<'_>) -> Option<Vec<String>> {
    let (call_id, AstKind::CallExpression(call)) = parent(reference, semantic)? else {
        return None;
    };
    if call.optional
        || !call
            .callee
            .span()
            .contains_inclusive(semantic.nodes().kind(reference).span())
    {
        return None;
    }
    let (await_id, AstKind::AwaitExpression(_)) = parent(call_id, semantic)? else {
        return None;
    };
    let (_, AstKind::VariableDeclarator(declarator)) = parent(await_id, semantic)? else {
        return None;
    };
    let BindingPattern::ObjectPattern(object) = &declarator.id else {
        return None;
    };
    object_names(object)
}

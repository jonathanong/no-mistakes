use super::*;
use oxc_syntax::node::NodeId;

mod all;
mod loader;
mod member;
mod registry;

type Parent<'a> = (NodeId, AstKind<'a>);

/// Export names a dynamic `import()` provably consumes, or `None` when the
/// module namespace escapes or is read dynamically. Recognized forms are
/// documented in `docs/rules/production-exports-only-used-by-tests.md`.
pub(super) fn proven(
    node: NodeId,
    semantic: &Semantic<'_>,
    calls: &mut Vec<String>,
) -> Option<Vec<String>> {
    let (id, kind) = parent(node, semantic)?;
    match kind {
        AstKind::StaticMemberExpression(member)
            if !member.optional && member.property.name == "then" =>
        {
            then_callback(member.span, id, semantic, calls)
        }
        AstKind::AwaitExpression(_) => awaited(id, semantic, calls),
        AstKind::ArrowFunctionExpression(function) => {
            registry::selected(id, function, semantic, calls)
                .or_else(|| loader::default(id, function, semantic))
        }
        AstKind::ArrayExpression(array) => all::selected(node, id, array, semantic),
        _ => None,
    }
}

fn parent<'a>(node: NodeId, semantic: &Semantic<'a>) -> Option<Parent<'a>> {
    semantic
        .nodes()
        .ancestors(node)
        .find(|n| {
            !matches!(
                n.kind(),
                AstKind::ParenthesizedExpression(_)
                    | AstKind::TSAsExpression(_)
                    | AstKind::TSSatisfiesExpression(_)
                    | AstKind::TSNonNullExpression(_)
                    | AstKind::TSInstantiationExpression(_)
                    | AstKind::TSTypeAssertion(_)
            )
        })
        .map(|n| (n.id(), n.kind()))
}

/// `import("x").then(cb)` where `cb` is `(m) => m.Name`, `(m) => ({ default: m.Name })` or `({ Name }) => ...`.
fn then_callback(
    member: oxc_span::Span,
    member_id: NodeId,
    semantic: &Semantic<'_>,
    calls: &mut Vec<String>,
) -> Option<Vec<String>> {
    let (_, AstKind::CallExpression(call)) = parent(member_id, semantic)? else {
        return None;
    };
    if call.optional || call.callee.span() != member || call.arguments.len() != 1 {
        return None;
    }
    let (params, body) = match call.arguments[0].as_expression()?.get_inner_expression() {
        Expression::ArrowFunctionExpression(f) => (
            &f.params,
            f.body
                .as_expression()
                .or_else(|| f.body.as_function_body().and_then(sole_return)),
        ),
        Expression::FunctionExpression(f) if !f.generator => {
            (&f.params, f.body.as_deref().and_then(sole_return))
        }
        _ => return None,
    };
    if params.items.len() != 1 || params.rest.is_some() {
        return None;
    }
    match &params.items[0].pattern {
        BindingPattern::ObjectPattern(object) => object_names(object),
        BindingPattern::BindingIdentifier(param) => {
            if matches!(
                body.map(Expression::get_inner_expression),
                Some(Expression::ObjectExpression(_))
            ) {
                return member::body(body?, &param.name).map(|name| vec![name]);
            }
            super::imports_namespace::runtime_members(semantic, param.symbol_id(), calls)
        }
        _ => None,
    }
}

fn sole_return<'a>(body: &'a FunctionBody<'a>) -> Option<&'a Expression<'a>> {
    let [Statement::ReturnStatement(statement)] = body.statements.as_slice() else {
        return None;
    };
    statement.argument.as_ref()
}

/// `const m = await import("x")` read only as `m.Name`, or a destructuring
/// `const { Name } = await import("x")`.
fn awaited(
    await_id: NodeId,
    semantic: &Semantic<'_>,
    calls: &mut Vec<String>,
) -> Option<Vec<String>> {
    if let Some((member_id, AstKind::StaticMemberExpression(member))) = parent(await_id, semantic) {
        if member.optional || super::imports_namespace::writes_member(semantic.nodes(), member_id) {
            return None;
        }
        let name = member.property.name.to_string();
        if super::imports_namespace::receiver_use(semantic.nodes(), member_id) {
            calls.push(name.clone());
        }
        return Some(vec![name]);
    }
    let (declarator_id, AstKind::VariableDeclarator(declarator)) = parent(await_id, semantic)?
    else {
        return None;
    };
    match &declarator.id {
        BindingPattern::ObjectPattern(object) => object_names(object),
        BindingPattern::BindingIdentifier(binding) => {
            // An exported binding escapes the module namespace.
            let (declaration, _) = parent(declarator_id, semantic)?;
            if matches!(
                parent(declaration, semantic)?.1,
                AstKind::ExportNamedDeclaration(_) | AstKind::ExportDeclaration(_)
            ) {
                return None;
            }
            members_read(binding.symbol_id.get()?, semantic, calls)
        }
        _ => None,
    }
}

fn members_read(
    symbol: oxc_syntax::symbol::SymbolId,
    semantic: &Semantic<'_>,
    calls: &mut Vec<String>,
) -> Option<Vec<String>> {
    for reference in semantic.scoping().get_resolved_references(symbol) {
        let AstKind::StaticMemberExpression(member) =
            semantic.nodes().parent_kind(reference.node_id())
        else {
            return None;
        };
        if member.optional {
            return None;
        }
    }
    super::imports_namespace::runtime_members(semantic, symbol, calls)
}

/// Keys of an object pattern; a rest element or computed key reads unknown names.
fn object_names(object: &ObjectPattern<'_>) -> Option<Vec<String>> {
    if object.rest.is_some() {
        return None;
    }
    let names = object
        .properties
        .iter()
        .map(|property| {
            if property.computed {
                return None;
            }
            crate::ast::property_key_name(&property.key).map(|name| name.into_owned())
        })
        .collect::<Option<Vec<_>>>()?;
    (!names.is_empty()).then_some(names)
}

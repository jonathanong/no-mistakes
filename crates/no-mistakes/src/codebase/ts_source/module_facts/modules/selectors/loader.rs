use super::*;
use oxc_syntax::operator::UnaryOperator;

/// `dynamic(() => import("x"))`: only an arrow passed directly as a call
/// argument is a loader whose result is the default export. A stored loader
/// (`const load = () => import("x")`) can be read in any way, so it is unproven.
pub(super) fn default(
    arrow: NodeId,
    function: &ArrowFunctionExpression<'_>,
    semantic: &Semantic<'_>,
) -> Option<Vec<String>> {
    let (call_id, AstKind::CallExpression(call)) = parent(arrow, semantic)? else {
        return None;
    };
    if !function.is_expression()
        || !call.arguments.iter().any(|a| a.span() == function.span)
        || mock_factory(call)
    {
        return None;
    }
    // Allow-list: the wrapper's result may be unwrapped, chained or forwarded by
    // any other parent, so only these sinks leave the module as its default.
    let proven = match parent(call_id, semantic)?.1 {
        AstKind::ExpressionStatement(_)
        | AstKind::ExportDefaultDeclaration(_)
        | AstKind::JSXExpressionContainer(_) => true,
        AstKind::VariableDeclarator(declarator) => stored_unconsumed(call_id, declarator, semantic),
        _ => false,
    };
    proven.then(|| vec!["default".to_string()])
}

/// `const X = wrap(...)` whose every reference is a JSX element name, a type
/// query or an export specifier. Reassigned, aliased or called uses are unproven.
fn stored_unconsumed(
    call_id: NodeId,
    declarator: &VariableDeclarator<'_>,
    semantic: &Semantic<'_>,
) -> bool {
    let BindingPattern::BindingIdentifier(binding) = &declarator.id else {
        return false;
    };
    let declaration = semantic
        .nodes()
        .parent_id(semantic.nodes().parent_id(call_id));
    let is_const = matches!(
        semantic.nodes().kind(declaration),
        AstKind::VariableDeclaration(d) if d.kind == VariableDeclarationKind::Const
    );
    is_const
        && semantic
            .scoping()
            .get_resolved_references(binding.symbol_id())
            .all(|reference| non_consuming(semantic.nodes().parent_kind(reference.node_id())))
}

fn non_consuming(kind: AstKind<'_>) -> bool {
    match kind {
        AstKind::JSXOpeningElement(_)
        | AstKind::JSXClosingElement(_)
        | AstKind::TSTypeQuery(_)
        | AstKind::ExportSpecifier(_) => true,
        AstKind::UnaryExpression(unary) => unary.operator == UnaryOperator::Typeof,
        _ => false,
    }
}

/// `vi.mock(path, async () => import("x"))` returns the whole module as the
/// replacement, a namespace use. It is not a lazy component wrapper, so it must
/// not be recorded as a use of `default`.
fn mock_factory(call: &CallExpression<'_>) -> bool {
    matches!(
        call.callee.get_inner_expression(),
        Expression::StaticMemberExpression(member)
            if matches!(
                member.property.name.as_str(),
                "mock" | "doMock" | "unstable_mockModule"
            )
    )
}

//! Spread-forwarding `mockImplementation` wrappers that add no root of their own.
use oxc_ast::{ast::*, AstKind};
use oxc_semantic::Semantic;

/// `vi.spyOn(x, "name").mockImplementation((...args) => original(...args))`: the
/// wrapper adds no root. Returns the literal spied name; a bare `vi.fn()` or a
/// computed name is not attributable to the mocked function, so it is not a pass-through.
pub(super) fn forwards_mock_arguments(
    semantic: &Semantic<'_>,
    node: oxc_syntax::node::NodeId,
    call: &CallExpression<'_>,
) -> Option<String> {
    let [Argument::SpreadElement(spread)] = call.arguments.as_slice() else {
        return None;
    };
    let Expression::Identifier(forwarded) = spread.argument.get_inner_expression() else {
        return None;
    };
    let forwarded_symbol = forwarded
        .reference_id
        .get()
        .and_then(|reference| semantic.scoping().get_reference(reference).symbol_id())?;
    // Any other use (an index write, `splice`, a nested closure) may change the arguments.
    let written = semantic
        .scoping()
        .get_resolved_references(forwarded_symbol)
        .count()
        != 1;
    let (function, params) =
        semantic
            .nodes()
            .ancestors(node)
            .find_map(|ancestor| match ancestor.kind() {
                AstKind::Function(function) => Some((ancestor.id(), &function.params)),
                AstKind::ArrowFunctionExpression(function) => {
                    Some((ancestor.id(), &function.params))
                }
                _ => None,
            })?;
    let rest_is_forwarded = !written
        && params.items.is_empty()
        && params.rest.as_ref().is_some_and(|rest| {
            matches!(&rest.rest.argument, BindingPattern::BindingIdentifier(id)
                if id.symbol_id.get() == Some(forwarded_symbol))
        });
    let AstKind::CallExpression(mock) = semantic.nodes().parent_kind(function) else {
        return None;
    };
    let Expression::StaticMemberExpression(member) = &mock.callee else {
        return None;
    };
    let mocks = matches!(
        member.property.name.as_str(),
        "mockImplementation" | "mockImplementationOnce"
    );
    (rest_is_forwarded && mocks)
        .then(|| spied_name(&member.object))
        .flatten()
}

/// The literal second argument of `spyOn(object, "name")`.
fn spied_name(expression: &Expression<'_>) -> Option<String> {
    let Expression::CallExpression(spy) = expression.get_inner_expression() else {
        return None;
    };
    let is_spy_on = match &spy.callee {
        Expression::Identifier(id) => id.name == "spyOn",
        Expression::StaticMemberExpression(member) => member.property.name == "spyOn",
        _ => false,
    };
    match spy
        .arguments
        .get(1)?
        .as_expression()?
        .get_inner_expression()
    {
        Expression::StringLiteral(name) if is_spy_on => {
            Some(crate::ast::js_literal(name).into_owned())
        }
        _ => None,
    }
}

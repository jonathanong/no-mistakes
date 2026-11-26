use super::super::super::TypeScriptModuleFacts;
use super::super::WriteScope;
use oxc_ast::{ast::*, AstKind};
use oxc_semantic::Semantic;
use oxc_span::GetSpan;
use oxc_syntax::node::NodeId;

pub(super) fn cleanup(
    semantic: &Semantic<'_>,
    facts: &TypeScriptModuleFacts,
    node: NodeId,
) -> Option<WriteScope> {
    let nodes = semantic.nodes();
    let member = nodes.parent_id(node);
    let AstKind::StaticMemberExpression(member_expression) = nodes.kind(member) else {
        return None;
    };
    if member_expression.property.span != nodes.kind(node).span()
        || !super::local_env::known_vi(semantic, facts, &member_expression.object)
    {
        return None;
    }
    let call = nodes.parent_id(member);
    let AstKind::CallExpression(expression) = nodes.kind(call) else {
        return None;
    };
    if expression.callee.span() != member_expression.span || !expression.arguments.is_empty() {
        return None;
    }
    let scope = super::scope::write_scope(semantic, call);
    matches!(scope, WriteScope::AfterEach(..) | WriteScope::AfterAll(..)).then_some(scope)
}

pub(super) fn callback(semantic: &Semantic<'_>, call: &CallExpression<'_>) -> Option<(u32, u32)> {
    if !matches!(
        super::scope::hook_name(semantic, &call.callee)?,
        "afterEach" | "afterAll"
    ) {
        return None;
    }
    call.arguments.iter().find_map(|argument| {
        match argument.as_expression()?.get_inner_expression() {
            Expression::ArrowFunctionExpression(function) => {
                Some((function.span.start, function.span.end))
            }
            Expression::FunctionExpression(function) => {
                Some((function.span.start, function.span.end))
            }
            _ => None,
        }
    })
}

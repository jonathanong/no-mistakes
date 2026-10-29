//! Where a `process.env` write sits in its enclosing code.
use super::super::WriteScope;
use oxc_ast::{ast::*, AstKind};
use oxc_span::GetSpan;
use oxc_syntax::node::NodeId;

/// The `Program` or `FunctionBody` whose statement list runs the expression
/// statement holding `node` unconditionally; `None` for a branch, loop, or
/// expression-position use.
fn unconditional_body(nodes: &oxc_semantic::AstNodes<'_>, node: NodeId) -> Option<NodeId> {
    let mut current = nodes.parent_id(node);
    if !matches!(nodes.kind(current), AstKind::ExpressionStatement(_)) {
        return None;
    }
    loop {
        current = nodes.parent_id(current);
        match nodes.kind(current) {
            AstKind::Program(_) | AstKind::FunctionBody(_) => return Some(current),
            AstKind::BlockStatement(_) => continue,
            _ => return None,
        }
    }
}

/// Whether the statement holding `node` runs whenever its enclosing body does.
pub(super) fn write_scope(semantic: &oxc_semantic::Semantic<'_>, node: NodeId) -> WriteScope {
    let nodes = semantic.nodes();
    let Some(body) = unconditional_body(nodes, node) else {
        return WriteScope::Conditional;
    };
    if matches!(nodes.kind(body), AstKind::Program(_)) {
        return WriteScope::Program;
    }
    let function = nodes.parent_id(body);
    let span = nodes.kind(function).span();
    let AstKind::CallExpression(registration) = nodes.parent_kind(function) else {
        return WriteScope::Function(span.start, span.end);
    };
    let hook = matches!(&registration.callee, Expression::Identifier(id)
        if matches!(id.name.as_str(), "beforeAll" | "beforeEach"));
    if !hook {
        return WriteScope::Function(span.start, span.end);
    }
    // A hook registered inside a branch may never run.
    if unconditional_body(nodes, nodes.parent_id(function)).is_none() {
        return WriteScope::Conditional;
    }
    let container = nodes
        .ancestors(function)
        .find(|ancestor| {
            matches!(
                ancestor.kind(),
                AstKind::Function(_) | AstKind::ArrowFunctionExpression(_) | AstKind::Program(_)
            )
        })
        .map_or((0, u32::MAX), |ancestor| {
            let span = ancestor.kind().span();
            (span.start, span.end)
        });
    WriteScope::Hook(container.0, container.1)
}

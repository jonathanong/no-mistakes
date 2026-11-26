//! Where a `process.env` write sits in its enclosing code.
use super::super::{ConditionalPlace, WriteScope};
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
        return WriteScope::Conditional(conditional_place(semantic, node));
    };
    if matches!(nodes.kind(body), AstKind::Program(_)) {
        return WriteScope::Program;
    }
    let function = nodes.parent_id(body);
    let span = nodes.kind(function).span();
    let AstKind::CallExpression(registration) = nodes.parent_kind(function) else {
        return WriteScope::Function(span.start, span.end);
    };
    let Some(hook) = hook_name(semantic, &registration.callee) else {
        return WriteScope::Function(span.start, span.end);
    };
    // A hook registered inside a branch may never run.
    if unconditional_body(nodes, nodes.parent_id(function)).is_none() {
        return WriteScope::Conditional(conditional_place(semantic, node));
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
    match hook {
        "beforeEach" => WriteScope::BeforeEach(container.0, container.1),
        "afterEach" => WriteScope::AfterEach(container.0, container.1),
        "afterAll" => WriteScope::AfterAll(container.0, container.1),
        _ => WriteScope::Hook(container.0, container.1),
    }
}

pub(super) fn hook_name(
    semantic: &oxc_semantic::Semantic<'_>,
    callee: &Expression<'_>,
) -> Option<&'static str> {
    let Expression::Identifier(id) = callee.get_inner_expression() else {
        return None;
    };
    let reference = semantic.scoping().get_reference(id.reference_id.get()?);
    let imported;
    let name = if let Some(symbol) = reference.symbol_id() {
        let declaration = semantic.scoping().symbol_declaration(symbol);
        let (AstKind::ImportSpecifier(binding), AstKind::ImportDeclaration(import)) = (
            semantic.nodes().kind(declaration),
            semantic.nodes().parent_kind(declaration),
        ) else {
            return None;
        };
        if binding.import_kind.is_type()
            || import.import_kind.is_type()
            || crate::ast::js_literal(&import.source) != "vitest"
        {
            return None;
        }
        imported = crate::ast::module_export_name(&binding.imported);
        imported.as_ref()
    } else {
        id.name.as_str()
    };
    match name {
        "beforeEach" => Some("beforeEach"),
        "beforeAll" => Some("beforeAll"),
        "afterEach" => Some("afterEach"),
        "afterAll" => Some("afterAll"),
        _ => None,
    }
}

mod branch_arm;

pub(super) fn conditional_place(
    semantic: &oxc_semantic::Semantic<'_>,
    node: NodeId,
) -> ConditionalPlace {
    let nodes = semantic.nodes();
    let branch = branch_arm::in_named_branch(nodes, node);
    let mut hook = None;
    let mut function = None;
    let mut current = node;
    loop {
        current = nodes.parent_id(current);
        let kind = nodes.kind(current);
        if hook.is_none() && function.is_none() {
            if let Some(span) = function_span(kind) {
                if let Some(container) = before_hook_span(semantic, nodes, current) {
                    hook = Some(container);
                } else {
                    function = Some(span);
                }
            }
        }
        if matches!(kind, AstKind::Program(_)) {
            break;
        }
    }
    ConditionalPlace {
        hook,
        function: if hook.is_some() { None } else { function },
        program: hook.is_none() && function.is_none(),
        branch,
    }
}

fn function_span(kind: AstKind<'_>) -> Option<(u32, u32)> {
    match kind {
        AstKind::Function(function) => Some((function.span.start, function.span.end)),
        AstKind::ArrowFunctionExpression(function) => {
            Some((function.span.start, function.span.end))
        }
        _ => None,
    }
}

fn before_hook_span(
    semantic: &oxc_semantic::Semantic<'_>,
    nodes: &oxc_semantic::AstNodes<'_>,
    function: NodeId,
) -> Option<(u32, u32)> {
    let AstKind::CallExpression(registration) = nodes.parent_kind(function) else {
        return None;
    };
    if !matches!(
        hook_name(semantic, &registration.callee),
        Some("beforeEach" | "beforeAll")
    ) {
        return None;
    }
    Some(
        nodes
            .ancestors(function)
            .find(|ancestor| {
                matches!(
                    ancestor.kind(),
                    AstKind::Function(_)
                        | AstKind::ArrowFunctionExpression(_)
                        | AstKind::Program(_)
                )
            })
            .map_or((0, u32::MAX), |ancestor| {
                let span = ancestor.kind().span();
                (span.start, span.end)
            }),
    )
}

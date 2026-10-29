//! References to the global `process` that could change `process.env` in a way
//! the write recorder does not name. Recall first: anything but a plain read poisons.
use super::super::PathProvenance;
use oxc_ast::{ast::*, AstKind};
use oxc_semantic::{AstNodes, Semantic};
use oxc_span::{GetSpan, Span};
use oxc_syntax::node::NodeId;
use oxc_syntax::operator::{BinaryOperator, UnaryOperator};

const PROCESS_MODULES: [&str; 2] = ["process", "node:process"];

pub(super) fn poison_process_imports(
    facts: &super::super::super::TypeScriptModuleFacts,
    out: &mut PathProvenance,
) {
    if facts
        .imports
        .iter()
        .any(|import| PROCESS_MODULES.contains(&import.specifier.as_str()))
    {
        out.env_unknown_write = true;
    }
}

/// `require("process")` and `import("process")`: another route to the real `env`.
pub(super) fn poison_process_loads(source: &str, out: &mut PathProvenance) {
    if PROCESS_MODULES.contains(&source) {
        out.env_unknown_write = true;
    }
}

/// `unstubAllEnvs` and the `unstubEnvs` config restore variables after a stub, and
/// `stubEnv` writes one. Only the direct `vi.stubEnv(...)` call is recorded by
/// name; an alias, destructuring, or computed key hides the write.
pub(super) fn poison_env_stubs(
    nodes: &AstNodes<'_>,
    node: NodeId,
    name: &str,
    out: &mut PathProvenance,
) {
    let poisons = match name {
        "unstubAllEnvs" | "unstubEnvs" => true,
        "stubEnv" => !is_direct_member_call(nodes, node),
        _ => false,
    };
    if poisons {
        out.env_unknown_write = true;
    }
}

fn is_direct_member_call(nodes: &AstNodes<'_>, name: NodeId) -> bool {
    let member = nodes.parent_id(name);
    let AstKind::StaticMemberExpression(expression) = nodes.kind(member) else {
        return false;
    };
    matches!(nodes.parent_kind(member),
        AstKind::CallExpression(call) if call.callee.span() == expression.span)
}

pub(super) fn from_identifier(
    semantic: &Semantic<'_>,
    node: NodeId,
    id: &IdentifierReference<'_>,
    out: &mut PathProvenance,
) {
    let unresolved = id.reference_id.get().is_some_and(|reference| {
        semantic
            .scoping()
            .get_reference(reference)
            .symbol_id()
            .is_none()
    });
    if !unresolved {
        return;
    }
    let nodes = semantic.nodes();
    let safe = match id.name.as_str() {
        "process" => process_use_is_safe(nodes, node, id.span),
        "globalThis" | "global" | "self" | "window" => !reaches_process(nodes, node, id.span),
        _ => true,
    };
    if !safe {
        out.env_unknown_write = true;
    }
}

fn literal_key(expression: &Expression<'_>) -> Option<String> {
    match expression.get_inner_expression() {
        Expression::StringLiteral(key) => Some(crate::ast::js_literal(key).into_owned()),
        _ => None,
    }
}

fn reaches_process(nodes: &AstNodes<'_>, node: NodeId, span: Span) -> bool {
    match nodes.parent_kind(node) {
        AstKind::StaticMemberExpression(member) => {
            member.object.span() == span && member.property.name == "process"
        }
        AstKind::ComputedMemberExpression(member) => {
            member.object.span() == span
                && literal_key(&member.expression).is_none_or(|key| key == "process")
        }
        _ => false,
    }
}

/// `process.cwd()`, `typeof process`, and `process.env` used only as a plain read.
fn process_use_is_safe(nodes: &AstNodes<'_>, node: NodeId, span: Span) -> bool {
    let parent = nodes.parent_id(node);
    match nodes.kind(parent) {
        AstKind::StaticMemberExpression(member) if member.object.span() == span => {
            member.property.name != "env" || env_use_is_safe(nodes, parent)
        }
        AstKind::ComputedMemberExpression(member) if member.object.span() == span => {
            match literal_key(&member.expression) {
                // `process["env"]` is a spelling the write recorder does not follow.
                Some(key) => key != "env",
                None => false,
            }
        }
        AstKind::UnaryExpression(unary) => unary.operator == UnaryOperator::Typeof,
        _ => false,
    }
}

/// How the `process.env` object itself is used: only member reads (writes are
/// recorded separately), copies by spread, and destructuring reads are safe.
fn env_use_is_safe(nodes: &AstNodes<'_>, env: NodeId) -> bool {
    let span = nodes.kind(env).span();
    let parent = nodes.parent_id(env);
    match nodes.kind(parent) {
        AstKind::StaticMemberExpression(member) if member.object.span() == span => {
            !is_pattern_write(nodes, parent)
        }
        AstKind::ComputedMemberExpression(member) if member.object.span() == span => {
            !is_pattern_write(nodes, parent)
        }
        AstKind::SpreadElement(_) => {
            matches!(nodes.parent_kind(parent), AstKind::ObjectExpression(_))
        }
        AstKind::VariableDeclarator(declarator) => {
            matches!(declarator.id, BindingPattern::ObjectPattern(_))
                && declarator
                    .init
                    .as_ref()
                    .is_some_and(|init| init.span() == span)
        }
        AstKind::BinaryExpression(binary) => binary.operator == BinaryOperator::In,
        _ => false,
    }
}

/// A member used as a destructuring, loop, or update target mutates the variable.
fn is_pattern_write(nodes: &AstNodes<'_>, member: NodeId) -> bool {
    matches!(
        nodes.parent_kind(member),
        AstKind::ArrayAssignmentTarget(_)
            | AstKind::ObjectAssignmentTarget(_)
            | AstKind::AssignmentTargetRest(_)
            | AstKind::AssignmentTargetWithDefault(_)
            | AstKind::AssignmentTargetPropertyProperty(_)
            | AstKind::UpdateExpression(_)
            | AstKind::ForInStatement(_)
            | AstKind::ForOfStatement(_)
    )
}

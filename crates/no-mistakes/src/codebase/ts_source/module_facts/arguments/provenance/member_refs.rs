//! Member accesses that reach code or the process object without naming them.
use super::super::PathProvenance;
use oxc_ast::ast::*;
use oxc_ast::AstKind;

/// Members of the global object that are plain host APIs and cannot reach code.
pub(super) const GLOBAL_SAFE: [&str; 20] = [
    "fetch",
    "console",
    "setTimeout",
    "setInterval",
    "clearTimeout",
    "clearInterval",
    "queueMicrotask",
    "structuredClone",
    "performance",
    "URL",
    "URLSearchParams",
    "Buffer",
    "crypto",
    "document",
    "location",
    "navigator",
    "localStorage",
    "sessionStorage",
    "requestAnimationFrame",
    "IS_REACT_ACT_ENVIRONMENT",
];

/// Members of `process` that are plain values or calls with no loader behavior.
pub(super) const PROCESS_SAFE: [&str; 14] = [
    "cwd",
    "platform",
    "argv",
    "pid",
    "version",
    "versions",
    "arch",
    "hrtime",
    "nextTick",
    "uptime",
    "exitCode",
    "memoryUsage",
    "stdout",
    "stderr",
];

/// `constructor` and `__proto__` reach the `Function` constructor and so any code.
fn reaches_constructor(name: &str) -> bool {
    matches!(name, "constructor" | "__proto__")
}

/// The literal text anywhere in a file, since `Reflect.get(f, "constructor")` and
/// `const k = "constructor"; f[k]` carry it without a member access.
pub(super) fn from_literal(text: &str, out: &mut PathProvenance) {
    if reaches_constructor(text) {
        out.env_unknown_write = true;
    }
}

/// The name as a property or binding key, except a class method named `constructor`.
pub(super) fn from_name(
    nodes: &oxc_semantic::AstNodes<'_>,
    node: oxc_semantic::NodeId,
    name: &str,
    out: &mut PathProvenance,
) {
    if reaches_constructor(name) && !matches!(nodes.parent_kind(node), AstKind::MethodDefinition(_))
    {
        out.env_unknown_write = true;
    }
}

/// A computed key that spells one of those properties, or builds a string that may.
pub(super) fn from_computed(member: &ComputedMemberExpression<'_>, out: &mut PathProvenance) {
    let unsafe_key = match member.expression.get_inner_expression() {
        Expression::StringLiteral(key) => {
            reaches_constructor(&crate::ast::js_literal(key))
        }
        Expression::BinaryExpression(binary) => {
            let text = |side: &Expression<'_>| {
                matches!(
                    side.get_inner_expression(),
                    Expression::StringLiteral(_) | Expression::TemplateLiteral(_)
                )
            };
            text(&binary.left) || text(&binary.right)
        }
        Expression::TemplateLiteral(template) => !template.expressions.is_empty(),
        Expression::CallExpression(_)
        | Expression::ConditionalExpression(_)
        | Expression::LogicalExpression(_)
        | Expression::SequenceExpression(_)
        | Expression::AwaitExpression(_) => true,
        _ => false,
    };
    if unsafe_key {
        out.env_unknown_write = true;
    }
}

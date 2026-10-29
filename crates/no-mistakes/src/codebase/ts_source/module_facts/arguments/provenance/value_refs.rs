//! Functions referenced other than as the callee of a direct call. A caller the
//! graph cannot see (`.then(sync)`, `.call`, `export { sync }`) may pass anything.
use super::super::PathProvenance;
use oxc_ast::{ast::*, AstKind};
use oxc_semantic::{AstNodes, Semantic};
use oxc_span::{GetSpan, Span};
use oxc_syntax::node::NodeId;

fn is_callee(nodes: &AstNodes<'_>, node: NodeId, span: Span) -> bool {
    matches!(nodes.parent_kind(node), AstKind::CallExpression(call) if call.callee.span() == span)
}

/// A symbol that may hold a function, plus the property names it came from when
/// it was destructured. Recall first: a binding whose origin cannot be proven
/// (`require`, `import()`, a destructured parameter) counts as a function.
fn function_like(semantic: &Semantic<'_>, id: &IdentifierReference<'_>) -> Option<Vec<String>> {
    let symbol = id
        .reference_id
        .get()
        .and_then(|reference| semantic.scoping().get_reference(reference).symbol_id())?;
    let mut keys = Vec::new();
    let function = match semantic
        .nodes()
        .kind(semantic.scoping().symbol_declaration(symbol))
    {
        AstKind::Function(_)
        | AstKind::ImportSpecifier(_)
        | AstKind::ImportDefaultSpecifier(_)
        | AstKind::ImportNamespaceSpecifier(_) => true,
        AstKind::FormalParameter(parameter) => pattern_keys(&parameter.pattern, &mut keys),
        AstKind::VariableDeclarator(declarator) => {
            let destructured = pattern_keys(&declarator.id, &mut keys);
            destructured
                || declarator.init.as_ref().is_some_and(|init| {
                    matches!(
                        init.get_inner_expression(),
                        Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_)
                    ) || loaded(init)
                })
        }
        _ => false,
    };
    function.then_some(keys)
}

/// Collect the property names of an object pattern; true when it is one.
fn pattern_keys(pattern: &BindingPattern<'_>, keys: &mut Vec<String>) -> bool {
    let BindingPattern::ObjectPattern(object) = pattern else {
        return false;
    };
    for property in &object.properties {
        keys.extend(
            property
                .key
                .name()
                .and_then(|name| name.as_str().map(str::to_string)),
        );
        pattern_keys(&property.value, keys);
    }
    true
}

/// An expression rooted at `require(..)` or `import(..)`.
fn loaded(expression: &Expression<'_>) -> bool {
    match expression.get_inner_expression() {
        Expression::AwaitExpression(await_) => loaded(&await_.argument),
        Expression::ImportExpression(_) => true,
        Expression::CallExpression(call) => {
            matches!(&call.callee, Expression::Identifier(id) if id.name == "require")
                || loaded_member(&call.callee)
        }
        other => loaded_member(other),
    }
}

fn loaded_member(expression: &Expression<'_>) -> bool {
    match expression.get_inner_expression() {
        Expression::StaticMemberExpression(member) => loaded(&member.object),
        Expression::ComputedMemberExpression(member) => loaded(&member.object),
        _ => false,
    }
}

pub(super) fn from_identifier(
    semantic: &Semantic<'_>,
    node: NodeId,
    id: &IdentifierReference<'_>,
    out: &mut PathProvenance,
) {
    if is_callee(semantic.nodes(), node, id.span) {
        return;
    }
    if let Some(keys) = function_like(semantic, id) {
        out.value_refs.insert(id.name.to_string());
        out.value_refs.extend(keys);
    }
}

pub(super) fn from_member(
    nodes: &AstNodes<'_>,
    node: NodeId,
    member: &StaticMemberExpression<'_>,
    out: &mut PathProvenance,
) {
    if !is_callee(nodes, node, member.span) {
        out.value_refs.insert(member.property.name.to_string());
    }
}

/// An import under another local name escapes under its exported name too.
pub(super) fn add_imported_names(
    facts: &super::super::super::TypeScriptModuleFacts,
    out: &mut PathProvenance,
) {
    for import in &facts.imports {
        // A bare package or builtin default is not a function of this repository.
        let external =
            import.specifier.starts_with("node:") || !import.specifier.contains(['/', '.']);
        for binding in &import.bindings {
            if out.value_refs.contains(&binding.local)
                && !(external && binding.imported == "default")
            {
                out.value_refs.insert(binding.imported.clone());
            }
        }
    }
}

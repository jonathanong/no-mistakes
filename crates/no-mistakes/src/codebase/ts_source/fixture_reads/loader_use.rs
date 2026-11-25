use super::FixtureReads;
use oxc_ast::{ast::Argument, AstKind};
use oxc_semantic::{NodeId, Semantic};
use oxc_span::GetSpan;
use std::collections::BTreeSet;

pub(super) fn has_unsafe_loader_use(semantic: &Semantic<'_>, reads: &FixtureReads) -> bool {
    let callees: BTreeSet<&[String]> = reads
        .calls
        .values()
        .flatten()
        .map(|proven| proven.callee.as_slice())
        .collect();
    if callees.is_empty() {
        return false;
    }
    let roots: BTreeSet<&str> = callees.iter().map(|callee| callee[0].as_str()).collect();
    for node in semantic.nodes().iter() {
        match node.kind() {
            AstKind::ImportSpecifier(spec)
                if roots.contains(spec.imported.name().as_str())
                    && spec.local.name.as_str() != spec.imported.name().as_str() =>
            {
                return true;
            }
            AstKind::IdentifierReference(ident) if roots.contains(ident.name.as_str()) => {
                let flags = semantic
                    .scoping()
                    .get_reference(ident.reference_id())
                    .flags();
                if !flags.is_value() || flags.is_value_as_type() {
                    continue;
                }
                let Some(callee) =
                    direct_literal_loader_callee(semantic, node.id(), ident.name.as_str())
                else {
                    return true;
                };
                if !callees.contains(callee.as_slice()) {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

fn direct_literal_loader_callee(
    semantic: &Semantic<'_>,
    mut node: NodeId,
    root: &str,
) -> Option<Vec<String>> {
    let nodes = semantic.nodes();
    let mut chain = vec![root.to_owned()];
    loop {
        let span = nodes.kind(node).span();
        let parent = nodes.parent_node(node);
        match parent.kind() {
            AstKind::StaticMemberExpression(member)
                if !member.optional && member.object.span() == span =>
            {
                chain.push(member.property.name.to_string());
                node = parent.id();
            }
            AstKind::CallExpression(call)
                if !call.optional
                    && call.callee.span() == span
                    && call.arguments.len() == 1
                    && matches!(
                        call.arguments.first(),
                        Some(Argument::StringLiteral(arg)) if arg.value.as_str().is_some()
                    ) =>
            {
                return Some(chain);
            }
            _ => return None,
        }
    }
}

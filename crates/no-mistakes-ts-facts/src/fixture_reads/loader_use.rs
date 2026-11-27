use super::FixtureReads;
use imports::{property_key_name, LoaderNames};
use oxc_ast::{
    ast::{Argument, Expression},
    AstKind,
};
use oxc_semantic::{NodeId, Semantic};
use oxc_span::GetSpan;
use std::collections::BTreeSet;

mod imports;

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
    let names = LoaderNames::from_callees(semantic, &callees);
    for node in semantic.nodes().iter() {
        match node.kind() {
            AstKind::ImportExpression(_) => return true,
            AstKind::ExportAllDeclaration(decl) if !decl.export_kind.is_type() => return true,
            AstKind::ExportFromDeclaration(decl) if !decl.export_kind.is_type() => {
                if decl
                    .source
                    .value
                    .as_str()
                    .is_some_and(|source| names.modules.contains(source))
                    || decl.specifiers.iter().any(|spec| {
                        !spec.export_kind.is_type() && names.contains(spec.local.name().as_str())
                    })
                {
                    return true;
                }
            }
            AstKind::ImportDeclaration(decl)
                if !decl.import_kind.is_type() && names.has_unsafe_import(decl) =>
            {
                return true;
            }
            AstKind::BindingIdentifier(ident)
                if names.roots.contains(ident.name.as_str())
                    && !names.is_allowed_binding(semantic, node.id(), ident.name.as_str()) =>
            {
                return true;
            }
            AstKind::BindingProperty(prop)
                if property_key_name(&prop.key).is_some_and(|name| names.contains(name)) =>
            {
                return true;
            }
            AstKind::StaticMemberExpression(member) => {
                let name = member.property.name.as_str();
                if name == "require" || names.roots.contains(name) {
                    return true;
                }
            }
            AstKind::ComputedMemberExpression(member)
                if matches!(
                    member.expression.get_inner_expression(),
                    Expression::StringLiteral(lit)
                        if lit.value.as_str().is_some_and(|name| {
                            name == "require" || names.contains(name)
                        })
                ) =>
            {
                return true;
            }
            AstKind::IdentifierReference(ident) => {
                let flags = semantic
                    .scoping()
                    .get_reference(ident.reference_id())
                    .flags();
                if !flags.is_value() || flags.is_value_as_type() {
                    continue;
                }
                let name = ident.name.as_str();
                if matches!(name, "require" | "createRequire") {
                    return true;
                }
                if names.roots.contains(name) {
                    let Some(callee) = direct_literal_loader_callee(semantic, node.id(), name)
                    else {
                        return true;
                    };
                    if !callees.contains(callee.as_slice()) {
                        return true;
                    }
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

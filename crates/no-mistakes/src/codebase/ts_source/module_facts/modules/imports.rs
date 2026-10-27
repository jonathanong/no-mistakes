use super::*;
use oxc_syntax::symbol::SymbolId;
pub(super) fn collect(
    value: &ImportDeclaration<'_>,
    semantic: &Semantic<'_>,
    facts: &mut TypeScriptModuleFacts,
) {
    let type_only = value.import_kind.is_type();
    let bindings: Vec<ModuleImportBinding> = value
        .specifiers
        .iter()
        .flatten()
        .map(|specifier| {
            let (local, imported, kind, individual_type) = match specifier {
                ImportDeclarationSpecifier::ImportSpecifier(value) => (
                    &value.local,
                    name(&value.imported),
                    "named",
                    value.import_kind.is_type(),
                ),
                ImportDeclarationSpecifier::ImportDefaultSpecifier(value) => {
                    (&value.local, "default".into(), "default", false)
                }
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(value) => {
                    (&value.local, "*".into(), "namespace", false)
                }
            };
            let symbol = local.symbol_id.get().expect("semantic import binding");
            ModuleImportBinding {
                imported,
                local: local.name.to_string(),
                kind: kind.into(),
                type_only: type_only || individual_type,
                binding_id: symbol.index(),
                span: specifier.span().into(),
                namespace_members: (type_only
                    && matches!(
                        specifier,
                        ImportDeclarationSpecifier::ImportNamespaceSpecifier(_)
                    ))
                .then(|| namespace_members(semantic, symbol))
                .flatten(),
            }
        })
        .collect();
    facts.imports.push(ModuleImport {
        specifier: crate::ast::js_literal(&value.source).into_owned(),
        type_only: type_only
            || (!bindings.is_empty() && bindings.iter().all(|binding| binding.type_only)),
        span: value.span.into(),
        bindings,
        qualifier: None,
    });
}

/// The root export named by `import("x").Name.Rest`; `None` for a bare module type.
pub(super) fn qualifier_root(value: &TSImportType<'_>) -> Option<String> {
    let mut name = value.qualifier.as_ref()?;
    loop {
        match name {
            TSImportTypeQualifier::Identifier(identifier) => {
                return Some(identifier.name.to_string())
            }
            TSImportTypeQualifier::QualifiedName(qualified) => name = &qualified.left,
        }
    }
}

/// Members of a type namespace import, or `None` when any reference is not a
/// `ns.Member` left operand (e.g. `typeof ns`) or the namespace is unused.
fn namespace_members(semantic: &Semantic<'_>, symbol: SymbolId) -> Option<Vec<String>> {
    let mut members = Vec::new();
    for reference in semantic.scoping().get_resolved_references(symbol) {
        let node = reference.node_id();
        // An identifier reference can only be the left operand of a qualified name.
        let AstKind::TSQualifiedName(qualified) = semantic.nodes().parent_kind(node) else {
            return None;
        };
        members.push(qualified.right.name.to_string());
    }
    (!members.is_empty()).then_some(members)
}

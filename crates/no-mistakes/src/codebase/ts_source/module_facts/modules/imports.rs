use super::*;
pub(super) fn collect(value: &ImportDeclaration<'_>, facts: &mut TypeScriptModuleFacts) {
    let type_only = value.import_kind.is_type();
    let bindings = value
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
            ModuleImportBinding {
                imported,
                local: local.name.to_string(),
                kind: kind.into(),
                type_only: type_only || individual_type,
                binding_id: local
                    .symbol_id
                    .get()
                    .expect("semantic import binding")
                    .index(),
                span: specifier.span().into(),
            }
        })
        .collect();
    facts.imports.push(ModuleImport {
        specifier: value.source.value.to_string(),
        type_only,
        span: value.span.into(),
        bindings,
    });
}

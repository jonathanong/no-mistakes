use oxc_ast::ast::{ImportDeclarationSpecifier, Program, Statement};
use std::collections::BTreeMap;

#[derive(Clone)]
pub(super) struct ImportBinding {
    pub(super) source: String,
    pub(super) imported: String,
}

pub(super) fn import_bindings(program: &Program<'_>) -> BTreeMap<String, ImportBinding> {
    let mut bindings = BTreeMap::new();
    for statement in &program.body {
        let Statement::ImportDeclaration(import) = statement else {
            continue;
        };
        for specifier in import.specifiers.iter().flatten() {
            let (local, imported) = match specifier {
                ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                    (specifier.local.name.to_string(), "default".to_string())
                }
                ImportDeclarationSpecifier::ImportSpecifier(specifier) => (
                    specifier.local.name.to_string(),
                    specifier.imported.name().to_string(),
                ),
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                    (specifier.local.name.to_string(), "*".to_string())
                }
            };
            bindings.insert(
                local,
                ImportBinding {
                    source: import.source.value.to_string(),
                    imported,
                },
            );
        }
    }
    bindings
}

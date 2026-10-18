use super::super::options::EmbeddedSqlOptions;
use oxc_ast::ast::{ImportDeclarationSpecifier, ImportOrExportKind, Program, Statement};
use std::collections::HashSet;

pub(super) fn configured_namespaces(
    program: &Program<'_>,
    options: &EmbeddedSqlOptions,
) -> HashSet<String> {
    if options.import_specifier.is_empty() {
        return HashSet::new();
    }
    program
        .body
        .iter()
        .filter_map(|statement| match statement {
            Statement::ImportDeclaration(import)
                if import.import_kind != ImportOrExportKind::Type
                    && super::super::scoped_bindings::from_configured_module(
                        import.source.value.as_str(),
                        &options.import_specifier,
                    ) =>
            {
                import.specifiers.as_ref()
            }
            _ => None,
        })
        .flat_map(|specifiers| specifiers.iter())
        .filter_map(|specifier| match specifier {
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(namespace) => {
                Some(namespace.local.name.to_string())
            }
            _ => None,
        })
        .collect()
}

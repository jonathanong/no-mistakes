use crate::import_shape::is_runtime_import;
use crate::imports::collect_identifier_references;
use oxc_ast::ast::{ImportDeclarationSpecifier, Program};

pub(crate) use crate::codebase::storybook_mdx::extract_mdx_source;

#[derive(Debug, Clone, Default)]
pub(crate) struct StorybookFileFacts {
    pub(crate) used_runtime_imports: Vec<UsedRuntimeImport>,
    pub(crate) side_effect_imports: Vec<StorybookSideEffectImport>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct UsedRuntimeImport {
    pub(crate) source: String,
    pub(crate) imported: String,
    pub(crate) local: String,
    pub(crate) namespace: bool,
    pub(crate) line: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StorybookSideEffectImport {
    pub(crate) source: String,
    pub(crate) line: u32,
}

pub(crate) fn extract_program(source: &str, program: &Program<'_>) -> StorybookFileFacts {
    let referenced = collect_identifier_references(program);
    let mut used_runtime_imports = Vec::new();
    let mut side_effect_imports = Vec::new();
    for stmt in &program.body {
        let oxc_ast::ast::Statement::ImportDeclaration(import) = stmt else {
            continue;
        };
        if !is_runtime_import(import) {
            continue;
        }
        let Some(specifiers) = &import.specifiers else {
            side_effect_imports.push(StorybookSideEffectImport {
                source: import.source.value.as_str().to_string(),
                line: crate::codebase::ts_source::byte_offset_to_line(
                    source,
                    import.span.start as usize,
                ),
            });
            continue;
        };
        for specifier in specifiers {
            let Some(imported) = imported_name(specifier) else {
                continue;
            };
            let local = local_name(specifier);
            if !referenced.contains(local) {
                continue;
            }
            used_runtime_imports.push(UsedRuntimeImport {
                source: import.source.value.as_str().to_string(),
                imported,
                local: local.to_string(),
                namespace: matches!(
                    specifier,
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(_)
                ),
                line: crate::codebase::ts_source::byte_offset_to_line(
                    source,
                    import.span.start as usize,
                ),
            });
        }
    }
    StorybookFileFacts {
        used_runtime_imports,
        side_effect_imports,
    }
}

fn imported_name(specifier: &ImportDeclarationSpecifier<'_>) -> Option<String> {
    match specifier {
        ImportDeclarationSpecifier::ImportDefaultSpecifier(_) => Some("default".to_string()),
        ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => Some("*".to_string()),
        ImportDeclarationSpecifier::ImportSpecifier(specifier)
            if !specifier.import_kind.is_type() =>
        {
            Some(specifier.imported.name().to_string())
        }
        ImportDeclarationSpecifier::ImportSpecifier(_) => None,
    }
}

fn local_name<'a>(specifier: &'a ImportDeclarationSpecifier<'a>) -> &'a str {
    match specifier {
        ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
            specifier.local.name.as_ref()
        }
        ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
            specifier.local.name.as_ref()
        }
        ImportDeclarationSpecifier::ImportSpecifier(specifier) => specifier.local.name.as_ref(),
    }
}

#[cfg(test)]
mod tests;

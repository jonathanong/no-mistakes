use super::{expression::Evaluator, Module, Reader};
mod exports;
use oxc_ast::ast::{
    Declaration, ImportDeclarationSpecifier, Program, Statement, VariableDeclarationKind,
};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub(super) fn parse(
    reader: &mut Reader<'_>,
    path: &Path,
    source: &str,
    program: &Program<'_>,
) -> Module {
    let mut imports = BTreeMap::new();
    let mut wrappers = BTreeSet::new();
    for statement in &program.body {
        let Statement::ImportDeclaration(import) = statement else {
            continue;
        };
        if import.import_kind.is_type() {
            continue;
        }
        let specifier = crate::ast::js_literal(&import.source);
        let exports = reader.import(path, specifier.as_ref());
        for binding in import.specifiers.iter().flatten() {
            let (local, exported) = match binding {
                ImportDeclarationSpecifier::ImportSpecifier(binding) => {
                    if binding.import_kind.is_type() {
                        continue;
                    }
                    let name = crate::ast::module_export_name(&binding.imported).into_owned();
                    if specifier.as_ref() == "vitest/config"
                        && matches!(name.as_str(), "defineConfig" | "defineProject")
                    {
                        wrappers.insert(binding.local.name.to_string());
                    }
                    (binding.local.name.to_string(), name)
                }
                ImportDeclarationSpecifier::ImportDefaultSpecifier(binding) => {
                    (binding.local.name.to_string(), "default".into())
                }
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(binding) => {
                    imports.insert(
                        binding.local.name.to_string(),
                        exports.namespace(
                            path,
                            source[..import.span.start as usize]
                                .bytes()
                                .filter(|byte| *byte == b'\n')
                                .count()
                                + 1,
                        ),
                    );
                    continue;
                }
            };
            if let Some(value) = exports.exports.get(&exported) {
                imports.insert(local, value.clone());
            }
        }
    }
    let mut bindings = super::super::super::shared::top_level_object_bindings(program);
    let mut exported = BTreeSet::new();
    for statement in &program.body {
        let (declaration, export) = match statement {
            Statement::VariableDeclaration(declaration) => (Some(declaration.as_ref()), false),
            Statement::ExportDeclaration(export) => match &export.declaration {
                Declaration::VariableDeclaration(declaration) => (Some(declaration.as_ref()), true),
                _ => (None, false),
            },
            _ => (None, false),
        };
        if let Some(declaration) = declaration {
            for binding in &declaration.declarations {
                let oxc_ast::ast::BindingPattern::BindingIdentifier(name) = &binding.id else {
                    continue;
                };
                if declaration.kind != VariableDeclarationKind::Const {
                    bindings.remove(name.name.as_str());
                }
                if export {
                    exported.insert(name.name.to_string());
                }
            }
        }
    }
    let mut evaluator = Evaluator {
        path,
        source,
        bindings,
        imports,
        wrappers,
        visiting: BTreeSet::new(),
    };
    exports::parse(reader, path, source, program, &mut evaluator, exported)
}

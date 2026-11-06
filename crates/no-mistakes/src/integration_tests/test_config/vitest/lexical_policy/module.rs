use super::{expression::Evaluator, Kind, Reader, Value};
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
) -> BTreeMap<String, Value> {
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
                        Value {
                            kind: Kind::Object(exports.clone(), false),
                            file: path.to_path_buf(),
                            line: 1,
                        },
                    );
                    continue;
                }
            };
            if let Some(value) = exports.get(&exported) {
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
    let mut exports = BTreeMap::new();
    for name in exported {
        if let Some(expression) = evaluator.bindings.get(&name).copied() {
            exports.insert(name, evaluator.eval(expression));
        }
    }
    for statement in &program.body {
        match statement {
            Statement::ExportDefaultDeclaration(export) => {
                if let Some(expression) = export.declaration.as_expression() {
                    exports.insert("default".into(), evaluator.eval(expression));
                }
            }
            Statement::ExportNamedDeclaration(export) => {
                for specifier in &export.specifiers {
                    if export.export_kind.is_type() || specifier.export_kind.is_type() {
                        continue;
                    }
                    let local = crate::ast::module_export_name(&specifier.local);
                    if let Some(value) = evaluator.named(local.as_ref()) {
                        exports.insert(
                            crate::ast::module_export_name(&specifier.exported).into_owned(),
                            value,
                        );
                    }
                }
            }
            Statement::ExportFromDeclaration(export) => {
                let external = reader.import(path, crate::ast::js_literal(&export.source).as_ref());
                for specifier in &export.specifiers {
                    if export.export_kind.is_type() || specifier.export_kind.is_type() {
                        continue;
                    }
                    let local = crate::ast::module_export_name(&specifier.local);
                    if let Some(value) = external.get(local.as_ref()) {
                        exports.insert(
                            crate::ast::module_export_name(&specifier.exported).into_owned(),
                            value.clone(),
                        );
                    }
                }
            }
            _ => {}
        }
    }
    if super::mutations::contains_mutation(program) {
        for value in exports.values_mut() {
            *value = value.unknown();
        }
    }
    exports
}

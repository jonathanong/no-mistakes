use oxc_ast::ast::{BindingPattern, ImportDeclarationSpecifier, Program, Statement};
use std::collections::{BTreeMap, BTreeSet};

mod commonjs;
pub(super) use commonjs::direct_literal_require_binding;
use commonjs::{is_direct_commonjs_require, require_binding};

#[derive(Clone)]
pub(super) struct ImportBinding {
    pub(super) source: String,
    pub(super) imported: String,
}

pub(super) fn import_bindings(program: &Program<'_>) -> BTreeMap<String, ImportBinding> {
    let mut bindings = BTreeMap::new();
    for statement in &program.body {
        match statement {
            Statement::ImportDeclaration(import) => {
                if import.import_kind.is_type() {
                    continue;
                }
                for specifier in import.specifiers.iter().flatten() {
                    let (local, imported) = match specifier {
                        ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                            (specifier.local.name.to_string(), "default".to_string())
                        }
                        ImportDeclarationSpecifier::ImportSpecifier(specifier)
                            if specifier.import_kind.is_type() =>
                        {
                            continue;
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
            Statement::VariableDeclaration(declaration) => {
                for declarator in &declaration.declarations {
                    let Some(init) = declarator.init.as_ref() else {
                        continue;
                    };
                    let Some((source, imported)) = require_binding(init) else {
                        continue;
                    };
                    let is_vitest_namespace =
                        matches!(&declarator.id, BindingPattern::BindingIdentifier(_))
                            && imported == "default"
                            && is_direct_commonjs_require(init)
                            && is_commonjs_vitest_namespace_source(&source);
                    let imported = if is_vitest_namespace {
                        "*".to_string()
                    } else {
                        imported
                    };
                    commonjs_bindings(&declarator.id, source, imported, &mut bindings);
                }
            }
            _ => {}
        }
    }
    bindings
}

fn commonjs_bindings(
    pattern: &BindingPattern<'_>,
    source: String,
    imported: String,
    bindings: &mut BTreeMap<String, ImportBinding>,
) {
    match pattern {
        BindingPattern::BindingIdentifier(identifier) => {
            bindings.insert(
                identifier.name.to_string(),
                ImportBinding { source, imported },
            );
        }
        // A destructured direct require is equivalent to a static named import.
        // Aliases stay exact; computed/rest patterns remain intentionally dynamic.
        BindingPattern::ObjectPattern(object) if imported == "default" => {
            for property in &object.properties {
                if property.computed {
                    continue;
                }
                let Some(imported) =
                    crate::integration_tests::test_config::shared_literals::property_key_name(
                        &property.key,
                    )
                else {
                    continue;
                };
                let BindingPattern::BindingIdentifier(local) = &property.value else {
                    continue;
                };
                bindings.insert(
                    local.name.to_string(),
                    ImportBinding {
                        source: source.clone(),
                        imported,
                    },
                );
            }
        }
        _ => {}
    }
}

fn is_commonjs_vitest_namespace_source(source: &str) -> bool {
    source == "vitest/config"
}

/// Runtime module sources, including side-effect imports, re-exports, and
/// literal CommonJS `require` and `require.resolve` calls. Dynamic Vitest
/// setup values use this for a bounded helper-module closure.
pub(in crate::integration_tests::test_config::vitest) fn import_sources(
    program: &Program<'_>,
) -> BTreeSet<String> {
    let mut sources = program
        .body
        .iter()
        .filter_map(|statement| match statement {
            Statement::ImportDeclaration(import)
                if crate::fetch::import_shape::is_runtime_import(import) =>
            {
                Some(import.source.value.to_string())
            }
            Statement::ExportNamedDeclaration(export)
                if crate::fetch::import_shape::is_runtime_export(export) =>
            {
                export
                    .source
                    .as_ref()
                    .map(|source| source.value.to_string())
            }
            Statement::ExportAllDeclaration(export) if !export.export_kind.is_type() => {
                Some(export.source.value.to_string())
            }
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    // Reuse canonical dependency extraction for CommonJS semantics instead of
    // teaching this config-only parser a second require recognizer. A literal
    // `require.resolve` is also a runtime loader candidate when deleted.
    sources.extend(
        crate::codebase::dependencies::extract::extract_imports_from_program(program)
            .into_iter()
            .filter(|import| {
                matches!(
                    import.kind,
                    crate::codebase::dependencies::extract::ImportKind::Require
                        | crate::codebase::dependencies::extract::ImportKind::RequireResolve
                )
            })
            .map(|import| import.specifier),
    );
    sources
}

use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{Expression, ImportDeclarationSpecifier, Program, Statement};
use std::collections::{BTreeMap, BTreeSet};

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
                    let Some((source, imported)) = commonjs_require_binding(init) else {
                        continue;
                    };
                    let oxc_ast::ast::BindingPattern::BindingIdentifier(identifier) =
                        &declarator.id
                    else {
                        continue;
                    };
                    bindings.insert(
                        identifier.name.to_string(),
                        ImportBinding { source, imported },
                    );
                }
            }
            _ => {}
        }
    }
    bindings
}

/// Follow only a literal CommonJS `require` binding. It is equivalent to a
/// static ESM import for config parsing; computed and executable forms remain
/// dynamic and retain the conservative fallback path.
fn commonjs_require_binding(expression: &Expression<'_>) -> Option<(String, String)> {
    match unwrap_ts_wrappers(expression) {
        Expression::CallExpression(call) if matches!(&call.callee, Expression::Identifier(identifier) if identifier.name == "require") =>
        {
            let [oxc_ast::ast::Argument::StringLiteral(source)] = call.arguments.as_slice() else {
                return None;
            };
            Some((source.value.to_string(), "default".to_string()))
        }
        Expression::StaticMemberExpression(member) => commonjs_require_binding(&member.object)
            .map(|(source, _)| (source, member.property.name.to_string())),
        _ => None,
    }
}

/// Runtime module sources, including side-effect imports, re-exports, and
/// literal CommonJS `require` calls. Dynamic Vitest setup values use this for
/// a bounded helper-module closure.
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
    // teaching this config-only parser a second require recognizer.
    sources.extend(
        crate::codebase::dependencies::extract::extract_imports_from_program(program)
            .into_iter()
            .filter(|import| {
                matches!(
                    import.kind,
                    crate::codebase::dependencies::extract::ImportKind::Require
                )
            })
            .map(|import| import.specifier),
    );
    sources
}

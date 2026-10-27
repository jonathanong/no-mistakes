use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::integration_tests::types::{ConfigProject, Framework};
use oxc_ast::ast::{Expression, ImportDeclarationSpecifier, Program, Statement};
use std::path::Path;

/// Retain opaque-root diagnostics separately from ordinary discovery results.
/// The same prepared program supplies this classification; no source is reread.
pub(super) fn evidence_error(
    framework: Framework,
    path: &Path,
    program: &Program<'_>,
    projects: &Result<Vec<ConfigProject>, String>,
) -> Option<String> {
    if !projects.as_ref().is_ok_and(Vec::is_empty) {
        return None;
    }
    if framework == Framework::Vitest
        && crate::integration_tests::is_vitest_project_array_path(path)
        && static_empty_workspace(program)
    {
        return None;
    }
    Some(format!(
        "unsupported opaque {} config root",
        framework.as_str()
    ))
}

fn static_empty_workspace(program: &Program<'_>) -> bool {
    program.body.iter().any(|statement| {
        let Statement::ExportDefaultDeclaration(export) = statement else {
            return false;
        };
        let Some(expression) = export.declaration.as_expression() else {
            return false;
        };
        match unwrap_ts_wrappers(expression) {
            Expression::ArrayExpression(array) => array.elements.is_empty(),
            Expression::CallExpression(call) if call.arguments.len() == 1 => {
                call.arguments.first().and_then(|argument| argument.as_expression())
                    .is_some_and(|argument| matches!(unwrap_ts_wrappers(argument), Expression::ArrayExpression(array) if array.elements.is_empty()))
                    && public_workspace_call(&call.callee, program)
            }
            _ => false,
        }
    })
}

fn public_workspace_call(callee: &Expression<'_>, program: &Program<'_>) -> bool {
    let (name, namespace) = match unwrap_ts_wrappers(callee) {
        Expression::Identifier(identifier) => (identifier.name.as_str(), false),
        Expression::StaticMemberExpression(member) if member.property.name == "defineWorkspace" => {
            let Expression::Identifier(identifier) = unwrap_ts_wrappers(&member.object) else {
                return false;
            };
            (identifier.name.as_str(), true)
        }
        _ => return false,
    };
    program.body.iter().any(|statement| {
        let Statement::ImportDeclaration(import) = statement else {
            return false;
        };
        !import.import_kind.is_type()
            && matches!(
                crate::ast::js_literal(&import.source).as_ref(),
                "vitest" | "vitest/config"
            )
            && import
                .specifiers
                .iter()
                .flatten()
                .any(|specifier| match specifier {
                    ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                        !namespace
                            && !specifier.import_kind.is_type()
                            && crate::ast::module_export_name(&specifier.imported)
                                == "defineWorkspace"
                            && specifier.local.name == name
                    }
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                        namespace && specifier.local.name == name
                    }
                    _ => false,
                })
    })
}

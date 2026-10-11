use super::super::Ctx;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{Expression, ImportDeclarationSpecifier, Program, Statement};

pub(super) fn direct_root(program: &Program<'_>, ctx: &Ctx<'_, '_>) -> bool {
    program.body.iter().any(|statement| {
        let Statement::ExportDefaultDeclaration(export) = statement else {
            return false;
        };
        let Some(expression) = export.declaration.as_expression() else {
            return false;
        };
        match unwrap_ts_wrappers(expression) {
            Expression::ObjectExpression(_) => true,
            Expression::CallExpression(call) if call.arguments.len() == 1 => {
                call.arguments
                    .first()
                    .and_then(|argument| argument.as_expression())
                    .is_some_and(|argument| {
                        matches!(
                            unwrap_ts_wrappers(argument),
                            Expression::ObjectExpression(_)
                        )
                    })
                    && public_define_config(&call.callee, program, ctx)
            }
            _ => false,
        }
    })
}

fn public_define_config(callee: &Expression<'_>, program: &Program<'_>, ctx: &Ctx<'_, '_>) -> bool {
    let (name, namespace) = match unwrap_ts_wrappers(callee) {
        Expression::Identifier(identifier) => (identifier.name.as_str(), false),
        Expression::StaticMemberExpression(member) if member.property.name == "defineConfig" => {
            let Expression::Identifier(identifier) = unwrap_ts_wrappers(&member.object) else {
                return false;
            };
            (identifier.name.as_str(), true)
        }
        _ => return false,
    };
    if ctx.bindings.contains_key(name) || ctx.functions.contains_key(name) {
        return false;
    }
    let Some(binding) = ctx.imports.get(name) else {
        return false;
    };
    if !matches!(
        binding.source.as_str(),
        "@playwright/test" | "playwright/test"
    ) || binding.imported != if namespace { "*" } else { "defineConfig" }
    {
        return false;
    }
    program.body.iter().any(|statement| {
        let Statement::ImportDeclaration(import) = statement else {
            return false;
        };
        !import.import_kind.is_type()
            && import
                .specifiers
                .iter()
                .flatten()
                .any(|specifier| match specifier {
                    ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                        !namespace
                            && !specifier.import_kind.is_type()
                            && specifier.local.name == name
                    }
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                        namespace && specifier.local.name == name
                    }
                    _ => false,
                })
    })
}

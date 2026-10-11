use super::{Ctx, Expression, Program, Statement};

pub(super) fn public_config_call(
    callee: &Expression<'_>,
    program: &Program<'_>,
    ctx: &Ctx<'_, '_>,
) -> Option<String> {
    let (binding, exported) = match crate::codebase::ts_source::unwrap_ts_wrappers(callee) {
        Expression::Identifier(identifier) => {
            if ctx.bindings.contains_key(identifier.name.as_str())
                || ctx.functions.contains_key(identifier.name.as_str())
            {
                return None;
            }
            if !is_direct_esm_import(program, identifier.name.as_str()) {
                return None;
            }
            let binding = ctx.imports.get(identifier.name.as_str())?;
            (binding, binding.imported.as_str())
        }
        Expression::StaticMemberExpression(member) => {
            let Expression::Identifier(identifier) = &member.object else {
                return None;
            };
            if ctx.bindings.contains_key(identifier.name.as_str())
                || ctx.functions.contains_key(identifier.name.as_str())
            {
                return None;
            }
            if !is_direct_esm_import(program, identifier.name.as_str()) {
                return None;
            }
            let binding = ctx.imports.get(identifier.name.as_str())?;
            if binding.imported != "*" {
                return None;
            }
            (binding, member.property.name.as_str())
        }
        _ => return None,
    };
    if (binding.source == "vitest/config"
        && matches!(exported, "defineConfig" | "defineProject" | "mergeConfig"))
        || (binding.source == "vite" && matches!(exported, "defineConfig" | "mergeConfig"))
    {
        Some(exported.to_string())
    } else {
        None
    }
}

fn is_direct_esm_import(program: &Program<'_>, name: &str) -> bool {
    program.body.iter().any(|statement| {
        let Statement::ImportDeclaration(import) = statement else {
            return false;
        };
        !import.import_kind.is_type()
            && import.specifiers.iter().flatten().any(|specifier| {
                use oxc_ast::ast::ImportDeclarationSpecifier;
                match specifier {
                    ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                        !specifier.import_kind.is_type() && specifier.local.name == name
                    }
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                        specifier.local.name == name
                    }
                    ImportDeclarationSpecifier::ImportDefaultSpecifier(_) => false,
                }
            })
    })
}

use super::super::ImportBinding;
use super::{Ctx, Options};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use anyhow::Result;
use commonjs::commonjs_workspace_expression;
use oxc_ast::ast::{BindingPattern, Declaration, Expression, Program, Statement};

mod commonjs;

pub(crate) fn workspace_default_options(
    program: &Program<'_>,
    ctx: &mut Ctx<'_, '_>,
) -> Result<Vec<Options>> {
    workspace_exported_options(program, ctx, "default")
}

fn workspace_exported_options(
    program: &Program<'_>,
    ctx: &mut Ctx<'_, '_>,
    exported: &str,
) -> Result<Vec<Options>> {
    if exported == "default" {
        if let Some(expression) = commonjs_workspace_expression(program) {
            return workspace_expression_options(expression, ctx);
        }
    }
    let mut star_sources = Vec::new();
    for statement in &program.body {
        match statement {
            Statement::ExportDefaultDeclaration(export) if exported == "default" => {
                return export
                    .declaration
                    .as_expression()
                    .map(|expression| workspace_expression_options(expression, ctx))
                    .unwrap_or_else(|| Ok(Vec::new()));
            }
            Statement::ExportNamedDeclaration(export) if !export.export_kind.is_type() => {
                if let Some(declaration) = &export.declaration {
                    if let Some(expression) = exported_declaration_expression(declaration, exported)
                    {
                        return workspace_expression_options(expression, ctx);
                    }
                }
                for specifier in &export.specifiers {
                    if specifier.export_kind.is_type() || specifier.exported.name() != exported {
                        continue;
                    }
                    if let Some(source) = &export.source {
                        return imported_workspace_options(
                            &ImportBinding {
                                source: source.value.to_string(),
                                imported: specifier.local.name().to_string(),
                            },
                            ctx,
                        );
                    }
                    return workspace_local_options(specifier.local.name().as_str(), ctx);
                }
            }
            Statement::ExportAllDeclaration(export)
                if exported != "default"
                    && export.exported.is_none()
                    && !export.export_kind.is_type() =>
            {
                star_sources.push(export.source.value.to_string());
            }
            _ => {}
        }
    }
    for source in star_sources {
        let options = imported_workspace_options(
            &ImportBinding {
                source,
                imported: exported.to_string(),
            },
            ctx,
        )?;
        if !options.is_empty() {
            return Ok(options);
        }
    }
    Ok(Vec::new())
}

fn exported_declaration_expression<'a>(
    declaration: &'a Declaration<'a>,
    exported: &str,
) -> Option<&'a Expression<'a>> {
    let Declaration::VariableDeclaration(declaration) = declaration else {
        return None;
    };
    declaration.declarations.iter().find_map(|declarator| {
        matches!(&declarator.id, BindingPattern::BindingIdentifier(identifier) if identifier.name == exported)
            .then_some(declarator.init.as_ref())
            .flatten()
    })
}

fn workspace_expression_options(
    expression: &Expression<'_>,
    ctx: &mut Ctx<'_, '_>,
) -> Result<Vec<Options>> {
    match unwrap_ts_wrappers(expression) {
        Expression::ArrayExpression(_) => super::super::expression_options(expression, ctx),
        Expression::CallExpression(call) if is_define_workspace_call(&call.callee, ctx) => call
            .arguments
            .first()
            .and_then(|argument| argument.as_expression())
            .map(|argument| super::super::expression_options(argument, ctx))
            .unwrap_or_else(|| Ok(Vec::new())),
        Expression::Identifier(identifier) => {
            let name = identifier.name.to_string();
            if !ctx.local_seen.insert(name.clone()) {
                return Ok(Vec::new());
            }
            let result = workspace_local_options(&name, ctx);
            ctx.local_seen.remove(&name);
            result
        }
        _ => Ok(Vec::new()),
    }
}

fn workspace_local_options(name: &str, ctx: &mut Ctx<'_, '_>) -> Result<Vec<Options>> {
    if let Some(expression) = ctx.bindings.get(name).copied() {
        workspace_expression_options(expression, ctx)
    } else if let Some(import) = ctx.imports.get(name).cloned() {
        imported_workspace_options(&import, ctx)
    } else {
        Ok(Vec::new())
    }
}

fn is_define_workspace_call(callee: &Expression<'_>, ctx: &Ctx<'_, '_>) -> bool {
    match unwrap_ts_wrappers(callee) {
        Expression::Identifier(identifier) => ctx
            .imports
            .get(identifier.name.as_str())
            .is_some_and(|import| {
                import.imported == "defineWorkspace" && is_vitest_source(&import.source)
            }),
        Expression::StaticMemberExpression(member) => {
            let Expression::Identifier(namespace) = unwrap_ts_wrappers(&member.object) else {
                return false;
            };
            member.property.name == "defineWorkspace"
                && ctx
                    .imports
                    .get(namespace.name.as_str())
                    .is_some_and(|import| {
                        // A direct CommonJS require is namespace-like. ESM
                        // default imports remain deliberately unsupported.
                        import.imported == "*" && is_vitest_source(&import.source)
                    })
        }
        _ => false,
    }
}

fn is_vitest_source(source: &str) -> bool {
    matches!(source, "vitest" | "vitest/config")
}

fn imported_workspace_options(
    import: &ImportBinding,
    ctx: &mut Ctx<'_, '_>,
) -> Result<Vec<Options>> {
    let Some(path) = ctx.resolver.resolve(&import.source, ctx.path) else {
        return Ok(Vec::new());
    };
    if !ctx.seen.insert(path.clone()) {
        return Ok(Vec::new());
    }
    let result = match crate::integration_tests::runner_config::read_request_source(&path) {
        Err(_) => Ok(Vec::new()),
        Ok(source) => crate::integration_tests::runner_config::with_program(
            &path,
            &source,
            |program, source| {
                let mut local_seen = std::collections::BTreeSet::new();
                let mut object_seen = std::collections::BTreeSet::new();
                let mut nested = Ctx {
                    source,
                    bindings: crate::integration_tests::test_config::vitest::shared::top_level_object_bindings(program),
                    functions: super::super::top_level_function_bodies(program),
                    imports: super::super::import_bindings(program),
                    resolver: ctx.resolver,
                    path: &path,
                    seen: ctx.seen,
                    local_seen: &mut local_seen,
                    object_seen: &mut object_seen,
                };
                workspace_exported_options(program, &mut nested, &import.imported)
            },
        )?,
    };
    ctx.seen.remove(&path);
    result
}

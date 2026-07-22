use super::super::{
    import_bindings, root_spreads, shared, top_level_function_bodies, Ctx, ImportBinding,
};
use super::setup_dependencies;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::integration_tests::types::{VitestSetupDependency, VitestSetupField};
use commonjs::commonjs_setup_expression;
use oxc_ast::ast::{ArrayExpressionElement, Expression, Program, Statement};
use std::collections::BTreeSet;

mod commonjs;

/// Resolve literal setup values exported by an imported helper module. This
/// mirrors imported project parsing while keeping arbitrary code dynamic.
pub(super) fn imported_setup_dependencies(
    import: &ImportBinding,
    field: VitestSetupField,
    parent: &mut Ctx<'_, '_>,
) -> Option<Vec<VitestSetupDependency>> {
    let candidates = parent
        .resolver
        .resolution_candidates(&import.source, parent.path)
        .into_iter()
        .filter(|path| super::super::super::setup_resolution::is_runtime_module(path))
        .collect::<BTreeSet<_>>();
    let path = parent
        .resolver
        .resolve(&import.source, parent.path)
        .filter(|path| super::super::super::setup_resolution::is_runtime_module(path))?;
    if !parent.seen.insert(path.clone()) {
        return None;
    }
    let result = crate::integration_tests::runner_config::read_request_source(&path)
        .ok()
        .and_then(|source| {
            crate::integration_tests::runner_config::with_program(
                &path,
                &source,
                |program, source| {
                    let bindings = shared::top_level_object_bindings(program);
                    let mut local_seen = BTreeSet::new();
                    let mut object_seen = BTreeSet::new();
                    let mut ctx = Ctx {
                        source,
                        // `exported_setup_dependencies` needs the bindings to
                        // locate an exported local while nested object parsing
                        // needs an owned map in its context.
                        bindings: bindings.clone(),
                        functions: top_level_function_bodies(program),
                        imports: import_bindings(program),
                        resolver: parent.resolver,
                        path: &path,
                        seen: parent.seen,
                        local_seen: &mut local_seen,
                        object_seen: &mut object_seen,
                    };
                    exported_setup_dependencies(
                        program,
                        &bindings,
                        &import.imported,
                        field,
                        &mut ctx,
                    )
                },
            )
            .ok()
            .flatten()
        });
    parent.seen.remove(&path);
    result.map(|mut dependencies| {
        for dependency in &mut dependencies {
            dependency.trigger_paths.insert(path.clone());
            dependency.trigger_paths.extend(candidates.iter().cloned());
        }
        dependencies
    })
}

fn exported_setup_dependencies<'a>(
    program: &'a Program<'a>,
    bindings: &std::collections::BTreeMap<String, &'a Expression<'a>>,
    exported: &str,
    field: VitestSetupField,
    ctx: &mut Ctx<'_, '_>,
) -> Option<Vec<VitestSetupDependency>> {
    if let Some(expression) = exported_setup_expression(program, bindings, exported) {
        if is_static_setup_expression(expression, bindings, &mut BTreeSet::new()) {
            return Some(setup_dependencies(expression, field, ctx));
        }
        return None;
    }
    if let Some(import) = root_spreads::sourced_reexport(program, exported) {
        return imported_setup_dependencies(&import, field, ctx);
    }
    if let Some(import) = root_spreads::imported_reexport(program, exported) {
        return imported_setup_dependencies(&import, field, ctx);
    }
    for source in root_spreads::star_barrel_sources(program) {
        let import = ImportBinding {
            source: source.to_string(),
            imported: exported.to_string(),
        };
        if let Some(dependencies) = imported_setup_dependencies(&import, field, ctx) {
            return Some(dependencies);
        }
    }
    None
}

/// Imports only replace their use-site declaration when their exported value
/// is a literal setup string/array. Calls and other executable values remain
/// dynamic at the use site so their fallback ownership stays intact.
fn is_static_setup_expression(
    expression: &Expression<'_>,
    bindings: &std::collections::BTreeMap<String, &Expression<'_>>,
    seen: &mut BTreeSet<String>,
) -> bool {
    match unwrap_ts_wrappers(expression) {
        Expression::StringLiteral(_) => true,
        Expression::TemplateLiteral(template) if template.expressions.is_empty() => true,
        Expression::ArrayExpression(array) => array.elements.iter().all(|element| match element {
            ArrayExpressionElement::Elision(_) => true,
            ArrayExpressionElement::SpreadElement(spread) => {
                is_static_setup_expression(&spread.argument, bindings, seen)
            }
            _ => element
                .as_expression()
                .is_some_and(|expression| is_static_setup_expression(expression, bindings, seen)),
        }),
        Expression::Identifier(identifier) => {
            let name = identifier.name.to_string();
            if !seen.insert(name.clone()) {
                return false;
            }
            let static_value = bindings
                .get(&name)
                .is_some_and(|binding| is_static_setup_expression(binding, bindings, seen));
            seen.remove(&name);
            static_value
        }
        _ => false,
    }
}

fn exported_setup_expression<'a>(
    program: &'a Program<'a>,
    bindings: &std::collections::BTreeMap<String, &'a Expression<'a>>,
    exported: &str,
) -> Option<&'a Expression<'a>> {
    for statement in &program.body {
        let Statement::ExportNamedDeclaration(export) = statement else {
            continue;
        };
        if export.export_kind.is_type() || export.source.is_some() {
            continue;
        }
        for specifier in &export.specifiers {
            if !specifier.export_kind.is_type() && specifier.exported.name() == exported {
                return bindings.get(specifier.local.name().as_str()).copied();
            }
        }
        if let Some(oxc_ast::ast::Declaration::VariableDeclaration(declaration)) =
            &export.declaration
        {
            for declarator in &declaration.declarations {
                let oxc_ast::ast::BindingPattern::BindingIdentifier(identifier) = &declarator.id
                else {
                    continue;
                };
                if identifier.name == exported {
                    return declarator.init.as_ref();
                }
            }
        }
    }
    if exported == "default" {
        let expression = program.body.iter().find_map(|statement| {
            let Statement::ExportDefaultDeclaration(export) = statement else {
                return None;
            };
            let expression = export.declaration.as_expression()?;
            match expression {
                Expression::Identifier(identifier) => {
                    bindings.get(identifier.name.as_str()).copied()
                }
                _ => Some(expression),
            }
        });
        if expression.is_some() {
            return expression;
        }
    }
    commonjs_setup_expression(program, exported)
}

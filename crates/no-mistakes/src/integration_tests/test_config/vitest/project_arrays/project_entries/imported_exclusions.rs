use self::exports::{
    exported_array, exported_expression, exported_function_body, reexported_imports,
};
use super::super::{
    direct_literal_require_binding, import_bindings, shared,
    string_projects::{string_project_paths, string_project_roots},
    top_level_function_bodies, Ctx, ImportBinding,
};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{ArrayExpressionElement, Expression};
use std::collections::BTreeSet;
use std::path::PathBuf;

mod calls;
mod exports;

pub(in crate::integration_tests::test_config::vitest::project_arrays) struct GlobalStringProjectExclusions
{
    pub(in crate::integration_tests::test_config::vitest::project_arrays) paths: BTreeSet<PathBuf>,
    pub(in crate::integration_tests::test_config::vitest::project_arrays) roots: BTreeSet<PathBuf>,
}

/// Collect negated config strings through the same static array forms that
/// project entries accept. This prepass happens before any config is parsed,
/// so a negation nested in an imported/spread array applies to an outer
/// positive entry as well as to entries in that imported array.
pub(in crate::integration_tests::test_config::vitest::project_arrays) fn global_excluded_string_projects(
    elements: &[&ArrayExpressionElement<'_>],
    ctx: &mut Ctx<'_, '_>,
) -> GlobalStringProjectExclusions {
    let mut excluded = GlobalStringProjectExclusions {
        paths: BTreeSet::new(),
        roots: BTreeSet::new(),
    };
    for element in elements {
        extend_global_exclusions(element, ctx, &mut excluded);
    }
    excluded
}

pub(super) fn extend_global_exclusions(
    element: &ArrayExpressionElement<'_>,
    ctx: &mut Ctx<'_, '_>,
    excluded: &mut GlobalStringProjectExclusions,
) {
    let Some(expression) = element.as_expression() else {
        let ArrayExpressionElement::SpreadElement(spread) = element else {
            return;
        };
        extend_expression_exclusions(&spread.argument, ctx, excluded);
        return;
    };
    let Expression::StringLiteral(project_config) = unwrap_ts_wrappers(expression) else {
        extend_expression_exclusions(expression, ctx, excluded);
        return;
    };
    let Some(specifier) = project_config.value.as_str().strip_prefix('!') else {
        return;
    };
    excluded.paths.extend(string_project_paths(specifier, ctx));
    excluded.roots.extend(string_project_roots(specifier, ctx));
}

pub(super) fn extend_expression_exclusions(
    expression: &Expression<'_>,
    ctx: &mut Ctx<'_, '_>,
    excluded: &mut GlobalStringProjectExclusions,
) {
    match unwrap_ts_wrappers(expression) {
        Expression::ArrayExpression(array) => {
            for element in &array.elements {
                extend_global_exclusions(element, ctx, excluded);
            }
        }
        Expression::Identifier(identifier) => {
            let name = identifier.name.as_str();
            if !ctx.local_seen.insert(name.to_string()) {
                return;
            }
            if let Some(binding) = ctx.bindings.get(name).copied() {
                extend_expression_exclusions(binding, ctx, excluded);
            } else if let Some(import) = ctx.imports.get(name).cloned() {
                extend_imported_exclusions(&import, ctx, excluded);
            }
            ctx.local_seen.remove(name);
        }
        Expression::CallExpression(call) => {
            if let Some(import) = direct_literal_require_binding(expression) {
                extend_imported_exclusions(&import, ctx, excluded);
            } else if call.arguments.is_empty() {
                calls::extend_call_exclusions(&call.callee, ctx, excluded);
            }
        }
        _ => {}
    }
}

fn extend_imported_exclusions(
    import: &ImportBinding,
    parent: &mut Ctx<'_, '_>,
    excluded: &mut GlobalStringProjectExclusions,
) {
    let Some(path) = parent.resolver.resolve(&import.source, parent.path) else {
        return;
    };
    if !parent.seen.insert(path.clone()) {
        return;
    }
    let _ =
        crate::integration_tests::runner_config::read_request_source(&path).and_then(|source| {
            crate::integration_tests::runner_config::with_program(
                &path,
                &source,
                |program, source| {
                    let bindings = shared::top_level_object_bindings(program);
                    let imports = import_bindings(program);
                    let array = exported_array(program, &bindings, &import.imported);
                    let reexports = reexported_imports(program, &import.imported);
                    let mut local_seen = BTreeSet::new();
                    let mut object_seen = BTreeSet::new();
                    let mut ctx = Ctx {
                        source,
                        bindings,
                        functions: top_level_function_bodies(program),
                        imports,
                        resolver: parent.resolver,
                        path: &path,
                        seen: parent.seen,
                        local_seen: &mut local_seen,
                        object_seen: &mut object_seen,
                    };
                    if let Some(array) = array {
                        for element in &array.elements {
                            extend_global_exclusions(element, &mut ctx, excluded);
                        }
                    } else {
                        for reexport in reexports {
                            extend_imported_exclusions(&reexport, &mut ctx, excluded);
                        }
                    }
                },
            )
        });
    parent.seen.remove(&path);
}

pub(super) fn extend_imported_call_exclusions(
    import: &ImportBinding,
    parent: &mut Ctx<'_, '_>,
    excluded: &mut GlobalStringProjectExclusions,
) {
    let Some(path) = parent.resolver.resolve(&import.source, parent.path) else {
        return;
    };
    if !parent.seen.insert(path.clone()) {
        return;
    }
    let _ =
        crate::integration_tests::runner_config::read_request_source(&path).and_then(|source| {
            crate::integration_tests::runner_config::with_program(
                &path,
                &source,
                |program, source| {
                    let bindings = shared::top_level_object_bindings(program);
                    let imports = import_bindings(program);
                    let expression = exported_expression(program, &bindings, &import.imported);
                    let function = exported_function_body(program, &import.imported);
                    let reexports = reexported_imports(program, &import.imported);
                    let mut local_seen = BTreeSet::new();
                    let mut object_seen = BTreeSet::new();
                    let mut ctx = Ctx {
                        source,
                        bindings,
                        functions: top_level_function_bodies(program),
                        imports,
                        resolver: parent.resolver,
                        path: &path,
                        seen: parent.seen,
                        local_seen: &mut local_seen,
                        object_seen: &mut object_seen,
                    };
                    if let Some(expression) = expression {
                        calls::extend_callable_exclusions(expression, &mut ctx, excluded);
                    } else if let Some(function) = function {
                        calls::extend_function_body_exclusions(function, &mut ctx, excluded);
                    } else {
                        for reexport in reexports {
                            extend_imported_call_exclusions(&reexport, &mut ctx, excluded);
                        }
                    }
                },
            )
        });
    parent.seen.remove(&path);
}

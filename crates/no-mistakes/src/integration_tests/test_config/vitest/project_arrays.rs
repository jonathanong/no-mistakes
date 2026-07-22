use super::{shared, Options};
use crate::codebase::ts_resolver::ImportResolution;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use anyhow::Result;
use oxc_ast::ast::{
    ArrayExpression, ArrayExpressionElement, Expression, FunctionBody, ObjectExpression, Program,
    Statement,
};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

mod calls;
mod exports;
mod expression_options;
mod function_returns;
mod functions;
pub(in crate::integration_tests::test_config::vitest) mod imports;
mod member_helpers;
mod members;
mod objects;
mod project_entries;
mod root_options;
mod root_spreads;
mod string_projects;

use expression_options::{
    expression_options, expression_statement_options, helper_expression_options, imported_options,
};
use function_returns::body_return_options;
use functions::top_level_function_bodies;
use imports::{import_bindings, ImportBinding};
use project_entries::{
    flattened_project_elements, global_excluded_string_project_paths, selected_string_project_paths,
};
pub(super) use root_options::root_options;
use string_projects::{string_project_options_for_paths, string_project_paths};

type ExprMap<'a> = BTreeMap<String, &'a Expression<'a>>;
type FnMap<'a> = BTreeMap<String, &'a FunctionBody<'a>>;

pub(super) struct Ctx<'a, 'r> {
    source: &'a str,
    bindings: ExprMap<'a>,
    functions: FnMap<'a>,
    imports: BTreeMap<String, ImportBinding>,
    resolver: &'r dyn ImportResolution,
    path: &'r Path,
    seen: &'r mut BTreeSet<PathBuf>,
    local_seen: &'r mut BTreeSet<String>,
    object_seen: &'r mut BTreeSet<String>,
}

pub(super) fn project_options(
    program: &Program<'_>,
    object: &ObjectExpression<'_>,
    source: &str,
    path: &Path,
    _root: &Path,
    resolver: &dyn ImportResolution,
) -> Result<Vec<Options>> {
    project_options_inner(program, object, source, path, _root, resolver)
}

/// A `vitest.workspace.*` or `vitest.projects.*` file exports projects directly, rather than nesting
/// them below `test.projects`. Reuse the ordinary project-array interpreter so
/// imports, cycles, visible-universe filtering, and ordering remain identical.
pub(super) fn workspace_options(
    program: &Program<'_>,
    source: &str,
    path: &Path,
    resolver: &dyn ImportResolution,
) -> Result<Vec<Options>> {
    let mut seen = BTreeSet::new();
    let mut local_seen = BTreeSet::new();
    let mut object_seen = BTreeSet::new();
    let mut ctx = Ctx {
        source,
        bindings: shared::top_level_object_bindings(program),
        functions: top_level_function_bodies(program),
        imports: import_bindings(program),
        resolver,
        path,
        seen: &mut seen,
        local_seen: &mut local_seen,
        object_seen: &mut object_seen,
    };
    exports::workspace_default_options(program, &mut ctx)
}

fn project_options_inner(
    program: &Program<'_>,
    object: &ObjectExpression<'_>,
    source: &str,
    path: &Path,
    _root: &Path,
    resolver: &dyn ImportResolution,
) -> Result<Vec<Options>> {
    let mut seen = BTreeSet::new();
    let mut local_seen = BTreeSet::new();
    let mut object_seen = BTreeSet::new();
    let mut ctx = Ctx {
        source,
        bindings: shared::top_level_object_bindings(program),
        functions: top_level_function_bodies(program),
        imports: import_bindings(program),
        resolver,
        path,
        seen: &mut seen,
        local_seen: &mut local_seen,
        object_seen: &mut object_seen,
    };
    root_spreads::project_options(object, &mut ctx).map(|options| options.unwrap_or_default())
}

pub(super) fn array_options(
    projects: &ArrayExpression<'_>,
    ctx: &mut Ctx<'_, '_>,
) -> Result<Vec<Options>> {
    let elements = flattened_project_elements(projects, ctx);
    // Collect exclusions before parsing any referenced config. This makes
    // negation order-independent without moving a positive config ahead of
    // adjacent inline entries in the final project list.
    let string_paths = selected_string_project_paths(&elements, ctx);
    let mut excluded_paths = string_paths.excluded.clone();
    excluded_paths.extend(global_excluded_string_project_paths(&elements, ctx));
    let mut parsed_string_paths = BTreeSet::new();
    let mut options = Vec::new();
    for element in elements {
        match element {
            ArrayExpressionElement::SpreadElement(spread) => {
                for option in expression_options(&spread.argument, ctx)? {
                    let Some(path) = option.standalone_config_path.as_ref() else {
                        options.push(option);
                        continue;
                    };
                    if !excluded_paths.contains(path) && parsed_string_paths.insert(path.clone()) {
                        options.push(option);
                    }
                }
            }
            _ => {
                if let Some(expression) = element.as_expression() {
                    if let Expression::StringLiteral(project_config) =
                        unwrap_ts_wrappers(expression)
                    {
                        for path in string_project_paths(project_config.value.as_str(), ctx) {
                            if string_paths.included.contains(&path)
                                && !excluded_paths.contains(&path)
                                && parsed_string_paths.insert(path.clone())
                            {
                                options.extend(string_project_options_for_paths(
                                    std::iter::once(path),
                                    ctx,
                                )?);
                            }
                        }
                        continue;
                    }
                    if !shared::is_array_expression_reference(expression, &ctx.bindings) {
                        if let Some(option) = objects::expression_object_options(expression, ctx)? {
                            options.push(option);
                        }
                    }
                }
            }
        }
    }
    Ok(options)
}

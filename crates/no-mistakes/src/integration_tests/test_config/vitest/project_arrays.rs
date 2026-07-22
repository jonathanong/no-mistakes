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
mod imports;
mod member_helpers;
mod members;
mod objects;
mod root_options;
mod root_spreads;
mod string_projects;

use expression_options::{
    expression_options, expression_statement_options, helper_expression_options, imported_options,
};
use function_returns::body_return_options;
use functions::top_level_function_bodies;
use imports::{import_bindings, ImportBinding};
pub(super) use root_options::root_options;
use string_projects::string_project_options;

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
    let mut options = Vec::new();
    for element in &projects.elements {
        match element {
            ArrayExpressionElement::SpreadElement(spread) => {
                options.extend(expression_options(&spread.argument, ctx)?);
            }
            _ => {
                if let Some(expression) = element.as_expression() {
                    if let Expression::StringLiteral(project_config) =
                        unwrap_ts_wrappers(expression)
                    {
                        options.extend(string_project_options(project_config.value.as_str(), ctx)?);
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

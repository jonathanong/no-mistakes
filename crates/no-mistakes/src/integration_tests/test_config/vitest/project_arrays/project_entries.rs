use super::{string_projects::string_project_paths, Ctx};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{ArrayExpression, ArrayExpressionElement, Expression};
use std::collections::BTreeSet;
use std::path::PathBuf;

mod imported_exclusions;
pub(super) use imported_exclusions::global_excluded_string_project_paths;

pub(super) struct StringProjectPaths {
    pub(super) included: BTreeSet<PathBuf>,
    pub(super) excluded: BTreeSet<PathBuf>,
}

pub(super) fn selected_string_project_paths(
    elements: &[&ArrayExpressionElement<'_>],
    ctx: &Ctx<'_, '_>,
) -> StringProjectPaths {
    let mut included = BTreeSet::new();
    let mut excluded = BTreeSet::new();
    for element in elements {
        let Some(expression) = element.as_expression() else {
            continue;
        };
        let Expression::StringLiteral(project_config) = unwrap_ts_wrappers(expression) else {
            continue;
        };
        let specifier = project_config.value.as_str();
        let (specifier, paths) = match specifier.strip_prefix('!') {
            Some(pattern) => (pattern, &mut excluded),
            None => (specifier, &mut included),
        };
        paths.extend(string_project_paths(specifier, ctx));
    }
    included.retain(|path| !excluded.contains(path));
    StringProjectPaths { included, excluded }
}

pub(super) fn flattened_project_elements<'a>(
    projects: &'a ArrayExpression<'a>,
    ctx: &Ctx<'a, '_>,
) -> Vec<&'a ArrayExpressionElement<'a>> {
    let mut elements = Vec::new();
    let mut seen_arrays = BTreeSet::new();
    extend_flattened_project_elements(projects, ctx, &mut seen_arrays, &mut elements);
    elements
}

fn extend_flattened_project_elements<'a>(
    projects: &'a ArrayExpression<'a>,
    ctx: &Ctx<'a, '_>,
    seen_arrays: &mut BTreeSet<String>,
    elements: &mut Vec<&'a ArrayExpressionElement<'a>>,
) {
    for element in &projects.elements {
        let ArrayExpressionElement::SpreadElement(spread) = element else {
            elements.push(element);
            continue;
        };
        let expression = unwrap_ts_wrappers(&spread.argument);
        if let Expression::ArrayExpression(array) = expression {
            extend_flattened_project_elements(array, ctx, seen_arrays, elements);
            continue;
        }
        let Expression::Identifier(identifier) = expression else {
            elements.push(element);
            continue;
        };
        let name = identifier.name.to_string();
        let Some(binding) = ctx.bindings.get(&name).copied() else {
            elements.push(element);
            continue;
        };
        let Expression::ArrayExpression(array) = unwrap_ts_wrappers(binding) else {
            elements.push(element);
            continue;
        };
        if !seen_arrays.insert(name.clone()) {
            elements.push(element);
            continue;
        }
        extend_flattened_project_elements(array, ctx, seen_arrays, elements);
        seen_arrays.remove(&name);
    }
}

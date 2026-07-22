use super::super::imports::import_sources;
use super::super::{shared, Ctx};
use crate::codebase::ts_resolver::ImportResolution;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::integration_tests::types::{VitestSetupDependency, VitestSetupField};
use oxc_ast::ast::{ArrayExpressionElement, Expression, FunctionBody};
use oxc_ast_visit::{walk, Visit};
use oxc_span::GetSpan;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub(super) fn setup_dependencies(
    value: &Expression<'_>,
    field: VitestSetupField,
    ctx: &mut Ctx<'_, '_>,
) -> Vec<VitestSetupDependency> {
    let value = shared::expression_value(value, &ctx.bindings);
    if let Expression::Identifier(identifier) = unwrap_ts_wrappers(value) {
        if let Some(import) = ctx.imports.get(identifier.name.as_str()).cloned() {
            if let Some(dependencies) =
                super::setup_imports::imported_setup_dependencies(&import, field, ctx)
            {
                return dependencies;
            }
        }
    }
    match unwrap_ts_wrappers(value) {
        Expression::ArrayExpression(array) => array
            .elements
            .iter()
            .flat_map(|element| match element {
                ArrayExpressionElement::Elision(_) => Vec::new(),
                ArrayExpressionElement::SpreadElement(spread) => {
                    setup_dependencies(&spread.argument, field, ctx)
                }
                _ => element
                    .as_expression()
                    .map(|expression| setup_dependencies(expression, field, ctx))
                    .unwrap_or_default(),
            })
            .collect(),
        expression => vec![setup_dependency(expression, field, ctx)],
    }
}

fn setup_dependency(
    expression: &Expression<'_>,
    field: VitestSetupField,
    ctx: &Ctx<'_, '_>,
) -> VitestSetupDependency {
    let declaration_line =
        crate::codebase::ts_source::line_number(ctx.source, expression.span().start) as u32;
    let resolved_expression = shared::expression_value(expression, &ctx.bindings);
    let specifier = shared::optional_string(resolved_expression, ctx.source);
    let trigger_paths = if specifier.is_none() {
        dynamic_trigger_paths(expression, ctx)
    } else {
        BTreeSet::from([ctx.path.to_path_buf()])
    };
    VitestSetupDependency {
        field,
        specifier,
        resolved_path: None,
        resolution_base: ctx
            .path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf(),
        declaration_path: ctx.path.to_path_buf(),
        declaration_line,
        trigger_paths,
        resolver_candidate_paths: BTreeSet::new(),
        transitive_trigger_paths: BTreeSet::new(),
    }
}

const MAX_DYNAMIC_TRIGGER_MODULES: usize = 64;

/// Dynamic setup values are deliberately not executable. Follow only static
/// identifiers and import declarations so an edit to a local wrapper or an
/// imported helper still causes the owner's bounded fallback.
fn dynamic_trigger_paths(expression: &Expression<'_>, ctx: &Ctx<'_, '_>) -> BTreeSet<PathBuf> {
    let mut paths = BTreeSet::from([ctx.path.to_path_buf()]);
    let mut seen_identifiers = BTreeSet::new();
    let mut seen_modules = BTreeSet::new();
    collect_expression_triggers(
        expression,
        ctx,
        &mut paths,
        &mut seen_identifiers,
        &mut seen_modules,
    );
    paths
}

fn collect_expression_triggers(
    expression: &Expression<'_>,
    ctx: &Ctx<'_, '_>,
    paths: &mut BTreeSet<PathBuf>,
    seen_identifiers: &mut BTreeSet<String>,
    seen_modules: &mut BTreeSet<PathBuf>,
) {
    let mut references = IdentifierReferences::default();
    references.visit_expression(expression);
    collect_identifier_triggers(references.names, ctx, paths, seen_identifiers, seen_modules);
}

fn collect_body_triggers(
    body: &FunctionBody<'_>,
    ctx: &Ctx<'_, '_>,
    paths: &mut BTreeSet<PathBuf>,
    seen_identifiers: &mut BTreeSet<String>,
    seen_modules: &mut BTreeSet<PathBuf>,
) {
    let mut references = IdentifierReferences::default();
    references.visit_function_body(body);
    collect_identifier_triggers(references.names, ctx, paths, seen_identifiers, seen_modules);
}

fn collect_identifier_triggers(
    names: BTreeSet<String>,
    ctx: &Ctx<'_, '_>,
    paths: &mut BTreeSet<PathBuf>,
    seen_identifiers: &mut BTreeSet<String>,
    seen_modules: &mut BTreeSet<PathBuf>,
) {
    for name in names {
        if !seen_identifiers.insert(name.clone()) {
            continue;
        }
        if let Some(import) = ctx.imports.get(&name) {
            if let Some(path) = ctx.resolver.resolve(&import.source, ctx.path) {
                collect_import_module(path, ctx.resolver, paths, seen_modules);
            }
        }
        if let Some(binding) = ctx.bindings.get(&name) {
            collect_expression_triggers(binding, ctx, paths, seen_identifiers, seen_modules);
        }
        if let Some(body) = ctx.functions.get(&name) {
            collect_body_triggers(body, ctx, paths, seen_identifiers, seen_modules);
        }
    }
}

fn collect_import_module(
    path: PathBuf,
    resolver: &dyn ImportResolution,
    paths: &mut BTreeSet<PathBuf>,
    seen_modules: &mut BTreeSet<PathBuf>,
) {
    paths.insert(path.clone());
    if seen_modules.len() >= MAX_DYNAMIC_TRIGGER_MODULES || !seen_modules.insert(path.clone()) {
        return;
    }
    let Ok(source) = crate::integration_tests::runner_config::read_request_source(&path) else {
        return;
    };
    let _ = crate::integration_tests::runner_config::with_program(
        &path,
        &source,
        |program, _source| {
            for source in import_sources(program) {
                let Some(dependency) = resolver.resolve(&source, &path) else {
                    continue;
                };
                collect_import_module(dependency, resolver, paths, seen_modules);
            }
        },
    );
}

#[derive(Default)]
struct IdentifierReferences {
    names: BTreeSet<String>,
}

impl<'a> Visit<'a> for IdentifierReferences {
    fn visit_identifier_reference(&mut self, identifier: &oxc_ast::ast::IdentifierReference<'a>) {
        self.names.insert(identifier.name.to_string());
        walk::walk_identifier_reference(self, identifier);
    }
}

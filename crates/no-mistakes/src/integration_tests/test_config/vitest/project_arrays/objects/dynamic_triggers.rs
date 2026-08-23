use super::super::imports::import_sources;
use super::super::Ctx;
use crate::codebase::ts_resolver::ImportResolution;
use oxc_ast::ast::{Expression, FunctionBody};
use oxc_ast_visit::{walk, Visit};
use std::collections::BTreeSet;
use std::path::PathBuf;

const MAX_DYNAMIC_TRIGGER_MODULES: usize = 64;

/// Dynamic values follow only static bindings and imports, never execution.
pub(super) fn dynamic_trigger_paths(
    expression: &Expression<'_>,
    ctx: &Ctx<'_, '_>,
) -> BTreeSet<PathBuf> {
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
            paths.extend(
                ctx.resolver
                    .resolution_candidates(&import.source, ctx.path)
                    .into_iter()
                    .filter(|path| is_runtime_module(path)),
            );
            if let Some(path) = ctx
                .resolver
                .resolve(&import.source, ctx.path)
                .filter(|path| is_runtime_module(path))
            {
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
                paths.extend(
                    resolver
                        .resolution_candidates(&source, &path)
                        .into_iter()
                        .filter(|path| is_runtime_module(path)),
                );
                let Some(dependency) = resolver
                    .resolve(&source, &path)
                    .filter(|path| is_runtime_module(path))
                else {
                    continue;
                };
                collect_import_module(dependency, resolver, paths, seen_modules);
            }
        },
    );
}

fn is_runtime_module(path: &std::path::Path) -> bool {
    super::super::super::setup_resolution::is_runtime_module(path)
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

use super::{declarations::DeclarationOps, top_level_function_bodies, Context, ImportBinding};
use crate::integration_tests::test_config::shared;
use anyhow::Result;
use oxc_ast::ast::{ExportDefaultDeclarationKind, Program, Statement};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

type DefaultOptions<T> =
    fn(&ExportDefaultDeclarationKind<'_>, &mut Context<'_, '_>) -> Result<Vec<T>>;
type CommonJsOptions<T> =
    fn(&Program<'_>, &Statement<'_>, &str, &mut Context<'_, '_>) -> Option<Result<Vec<T>>>;
type ExportLookup<T> =
    fn(&Program<'_>, &str, &Path, &mut Context<'_, '_>, &str) -> Result<Option<Vec<T>>>;

pub(in crate::integration_tests::test_config) struct ExportScanOps<T> {
    pub declarations: DeclarationOps<T>,
    pub import_bindings: fn(&Program<'_>) -> BTreeMap<String, ImportBinding>,
    pub default_options: DefaultOptions<T>,
    pub commonjs_options: CommonJsOptions<T>,
    pub preserve_test_scope: bool,
}

pub(in crate::integration_tests::test_config) fn exported_options_lookup<T>(
    program: &Program<'_>,
    source: &str,
    path: &Path,
    parent: &mut Context<'_, '_>,
    exported: &str,
    ops: &ExportScanOps<T>,
    recurse: ExportLookup<T>,
) -> Result<Option<Vec<T>>> {
    let mut local_seen = BTreeSet::new();
    let mut object_seen = BTreeSet::new();
    let mut ctx = Context {
        is_test_object: ops.preserve_test_scope && parent.is_test_object,
        source,
        bindings: shared::top_level_object_bindings(program),
        functions: top_level_function_bodies(program),
        imports: (ops.import_bindings)(program),
        resolver: parent.resolver,
        path,
        seen: parent.seen,
        local_seen: &mut local_seen,
        object_seen: &mut object_seen,
    };
    let mut export_all_sources = Vec::new();
    for statement in &program.body {
        match statement {
            Statement::ExportNamedDeclaration(export) => {
                if export.export_kind.is_type() {
                    continue;
                }
                if let Some(options) = super::declarations::named_export_options(
                    export,
                    exported,
                    &mut ctx,
                    &ops.declarations,
                ) {
                    return options.map(Some);
                }
            }
            Statement::ExportDeclaration(export) => {
                if let Some(options) = super::declarations::declaration_options(
                    &export.declaration,
                    exported,
                    &mut ctx,
                    &ops.declarations,
                ) {
                    return options.map(Some);
                }
            }
            Statement::ExportFromDeclaration(export) => {
                if let Some(options) = super::declarations::export_from_options(
                    export,
                    exported,
                    &mut ctx,
                    &ops.declarations,
                ) {
                    return options.map(Some);
                }
            }
            Statement::ExportAllDeclaration(export)
                if exported != "default"
                    && export.exported.is_none()
                    && !export.export_kind.is_type() =>
            {
                export_all_sources.push(export.source.value.to_string());
            }
            Statement::ExportDefaultDeclaration(export) if exported == "default" => {
                return (ops.default_options)(&export.declaration, &mut ctx).map(Some);
            }
            Statement::ExpressionStatement(_) => {
                if let Some(options) =
                    (ops.commonjs_options)(program, statement, exported, &mut ctx)
                {
                    return options.map(Some);
                }
            }
            _ => {}
        }
    }
    super::resolve_imported_star_exports(export_all_sources, exported, path, &mut ctx, recurse)
}

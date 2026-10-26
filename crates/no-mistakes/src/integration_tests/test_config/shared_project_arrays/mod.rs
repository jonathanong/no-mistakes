use crate::codebase::ts_resolver::ImportResolution;
use anyhow::Result;
use oxc_ast::ast::{FunctionBody, Program};
use oxc_ast::ast::{ImportDeclaration, ImportDeclarationSpecifier};
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;

pub(super) mod calls;
mod context;
pub(super) mod declarations;
pub(super) mod export_scan;
mod exports;
mod functions;
mod member_ops;
pub(super) mod members;
pub(super) mod object_calls;
pub(super) mod object_members;
pub(super) use context::{Context, ExprMap, FnMap};
pub(super) use exports::{imported_reexport, named_export_object, sourced_reexport};
pub(super) use functions::top_level_function_bodies;

/// Imported config files use the request's source store. A failed read means the
/// imported project contributes no options; parser and projection errors still propagate.
pub(super) fn read_imported_program<T: Default>(
    path: &Path,
    evaluate: impl FnOnce(&Program<'_>, &str) -> Result<T>,
) -> Result<T> {
    match crate::integration_tests::runner_config::read_request_source(path) {
        Err(_) => Ok(T::default()),
        Ok(source) => {
            crate::integration_tests::runner_config::with_program(path, &source, evaluate)
                .and_then(|value| value)
        }
    }
}

/// An `export *` lookup is valid only when exactly one source resolves it.
/// Empty options still count as a resolved export, matching direct exports.
pub(super) fn resolve_star_exports<T, C>(
    sources: Vec<String>,
    exported: &str,
    path: &Path,
    ctx: &mut C,
    mut lookup: impl FnMut(&ImportBinding, &Path, &mut C) -> Result<Option<Vec<T>>>,
) -> Result<Option<Vec<T>>> {
    let mut resolved = None;
    for source in sources {
        let binding = ImportBinding {
            source,
            imported: exported.to_string(),
        };
        let Some(options) = lookup(&binding, path, ctx)? else {
            continue;
        };
        if resolved.is_some() {
            return Ok(None);
        }
        resolved = Some(options);
    }
    Ok(resolved)
}

pub(super) fn resolve_imported_star_exports<T>(
    sources: Vec<String>,
    exported: &str,
    path: &Path,
    ctx: &mut Context<'_, '_>,
    lookup: StarExportLookup<T>,
) -> Result<Option<Vec<T>>> {
    resolve_star_exports(sources, exported, path, ctx, |import, base_path, ctx| {
        with_imported_context(import, base_path, ctx, |program, source, path, ctx| {
            lookup(program, source, path, ctx, &import.imported)
        })
    })
}

type StarExportLookup<T> =
    fn(&Program<'_>, &str, &Path, &mut Context<'_, '_>, &str) -> Result<Option<Vec<T>>>;

/// Resolve an imported helper once, preserving the active import cycle guard
/// through parsing and projection. Unresolved and unreadable imports are empty.
pub(super) fn imported_function_options<T: Default>(
    import: &ImportBinding,
    base_path: &Path,
    ctx: &mut Context<'_, '_>,
    import_bindings: for<'a> fn(&'a Program<'a>) -> BTreeMap<String, ImportBinding>,
    evaluate: impl for<'a, 'r> FnOnce(&'a FunctionBody<'a>, &mut Context<'a, 'r>) -> Result<T>,
) -> Result<T> {
    let resolver = ctx.resolver;
    let is_test_object = ctx.is_test_object;
    let Some(path) = resolver.resolve(&import.source, base_path) else {
        return Ok(T::default());
    };
    if !ctx.seen.insert(path.clone()) {
        return Ok(T::default());
    }
    let result = read_imported_program(&path, |program, source| {
        let mut local_seen = BTreeSet::new();
        let mut object_seen = BTreeSet::new();
        let mut scoped = Context {
            is_test_object,
            source,
            bindings: crate::integration_tests::test_config::shared::top_level_object_bindings(
                program,
            ),
            functions: top_level_function_bodies(program),
            imports: import_bindings(program),
            resolver,
            path: &path,
            seen: ctx.seen,
            local_seen: &mut local_seen,
            object_seen: &mut object_seen,
        };
        let Some(body) = scoped.functions.get(import.imported.as_str()).copied() else {
            return Ok(T::default());
        };
        evaluate(body, &mut scoped)
    });
    ctx.seen.remove(&path);
    result
}

pub(super) trait ImportContext {
    fn resolver(&self) -> &dyn ImportResolution;
    fn seen(&mut self) -> &mut BTreeSet<PathBuf>;
}

pub(super) fn with_imported_context<C: ImportContext, T: Default>(
    import: &ImportBinding,
    base_path: &Path,
    ctx: &mut C,
    evaluate: impl FnOnce(&Program<'_>, &str, &Path, &mut C) -> Result<T>,
) -> Result<T> {
    let Some(path) = ctx.resolver().resolve(&import.source, base_path) else {
        return Ok(T::default());
    };
    if !ctx.seen().insert(path.clone()) {
        return Ok(T::default());
    }
    let result = read_imported_program(&path, |program, source| {
        evaluate(program, source, &path, ctx)
    });
    ctx.seen().remove(&path);
    result
}

#[derive(Clone)]
pub(super) struct ImportBinding {
    pub(super) source: String,
    pub(super) imported: String,
}

/// Collect runtime ESM bindings in statement order. Callers may add framework-specific
/// CommonJS bindings between imports without changing which declaration wins.
pub(super) fn add_es_import_bindings(
    import: &ImportDeclaration<'_>,
    bindings: &mut BTreeMap<String, ImportBinding>,
) {
    if import.import_kind.is_type() {
        return;
    }
    for specifier in import.specifiers.iter().flatten() {
        let (local, imported) = match specifier {
            ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                (specifier.local.name.to_string(), "default".to_string())
            }
            ImportDeclarationSpecifier::ImportSpecifier(specifier)
                if specifier.import_kind.is_type() =>
            {
                continue;
            }
            ImportDeclarationSpecifier::ImportSpecifier(specifier) => (
                specifier.local.name.to_string(),
                specifier.imported.name().to_string(),
            ),
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                (specifier.local.name.to_string(), "*".to_string())
            }
        };
        bindings.insert(
            local,
            ImportBinding {
                source: crate::ast::js_literal(&import.source).into_owned(),
                imported,
            },
        );
    }
}

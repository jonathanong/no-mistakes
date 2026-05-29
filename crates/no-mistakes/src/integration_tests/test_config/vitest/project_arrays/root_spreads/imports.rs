use super::{
    import_bindings, shared, top_level_function_bodies, Ctx, ImportBinding, Options, Scope,
};
use super::{imported_reexport, named_export_object, sourced_reexport, star_barrel_sources};
use crate::ast;
use anyhow::Result;
use oxc_ast::ast::Program;
use std::collections::BTreeSet;
use std::path::Path;

pub(super) fn imported_project_options(
    import: &ImportBinding,
    base_path: &Path,
    ctx: &mut Ctx<'_, '_>,
    scope: Scope,
) -> Result<Option<Vec<Options>>> {
    let Some(path) = ctx.resolver.resolve(&import.source, base_path) else {
        return Ok(None);
    };
    if !ctx.seen.insert(path.clone()) {
        return Ok(None);
    }
    let result = match std::fs::read_to_string(&path) {
        Err(_) => Ok(None),
        Ok(source) => ast::with_program(&path, &source, |program, source| {
            exported_project_options(program, source, import.imported.as_str(), &path, ctx, scope)
        })
        .and_then(|options| options),
    };
    ctx.seen.remove(&path);
    result
}

fn exported_project_options(
    program: &Program<'_>,
    source: &str,
    exported: &str,
    path: &Path,
    parent: &mut Ctx<'_, '_>,
    scope: Scope,
) -> Result<Option<Vec<Options>>> {
    let bindings = shared::top_level_object_bindings(program);
    let object = if exported == "default" {
        shared::default_export_object(program, &bindings)
            .or_else(|| named_export_object(program, exported, &bindings))
    } else {
        named_export_object(program, exported, &bindings)
    };
    let Some(object) = object else {
        if let Some(import) = sourced_reexport(program, exported) {
            return imported_project_options(&import, path, parent, scope);
        }
        if let Some(import) = imported_reexport(program, exported) {
            return imported_project_options(&import, path, parent, scope);
        }
        for source in star_barrel_sources(program) {
            let b = ImportBinding {
                source: source.to_string(),
                imported: exported.to_string(),
            };
            if let Some(r) = imported_project_options(&b, path, parent, scope)? {
                return Ok(Some(r));
            }
        }
        return Ok(None);
    };
    let mut local_seen = BTreeSet::new();
    let mut object_seen = BTreeSet::new();
    let mut ctx = Ctx {
        source,
        bindings,
        functions: top_level_function_bodies(program),
        imports: import_bindings(program),
        resolver: parent.resolver,
        path,
        seen: parent.seen,
        local_seen: &mut local_seen,
        object_seen: &mut object_seen,
    };
    super::project_options_in(object, &mut ctx, scope)
}

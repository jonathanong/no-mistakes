use super::super::{
    import_bindings, root_spreads, shared, top_level_function_bodies, Ctx, ImportBinding,
};
use super::setup_dependencies;
use crate::integration_tests::types::{VitestSetupDependency, VitestSetupField};
use oxc_ast::ast::{Expression, Program};
use std::collections::BTreeSet;

mod commonjs;
mod static_expression;

use static_expression::{exported_setup_expression, is_static_setup_expression};

/// Resolve literal setup values exported by an imported helper module. This
/// mirrors imported project parsing while keeping arbitrary code dynamic.
pub(super) fn imported_setup_dependencies(
    import: &ImportBinding,
    field: VitestSetupField,
    parent: &mut Ctx<'_, '_>,
) -> Option<Vec<VitestSetupDependency>> {
    imported_setup_dependencies_inner(import, None, field, parent)
}

pub(super) fn imported_setup_member_dependencies(
    import: &ImportBinding,
    member: &str,
    field: VitestSetupField,
    parent: &mut Ctx<'_, '_>,
) -> Option<Vec<VitestSetupDependency>> {
    imported_setup_dependencies_inner(import, Some(member), field, parent)
}

fn imported_setup_dependencies_inner(
    import: &ImportBinding,
    member: Option<&str>,
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
                        member,
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
            // The importing config is an ownership trigger even when the
            // literal setup declaration lives in a static helper module.
            dependency.trigger_paths.insert(parent.path.to_path_buf());
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
    member: Option<&str>,
    field: VitestSetupField,
    ctx: &mut Ctx<'_, '_>,
) -> Option<Vec<VitestSetupDependency>> {
    if let Some(mut expression) = exported_setup_expression(program, bindings, exported) {
        if let Some(member) = member {
            let object = super::expression_object(expression, bindings)?;
            expression = shared::property_expression_deep(object, member, bindings)?;
        }
        if is_static_setup_expression(expression, bindings, &mut BTreeSet::new()) {
            return Some(setup_dependencies(expression, field, ctx));
        }
        return None;
    }
    if let Some(import) = root_spreads::sourced_reexport(program, exported) {
        return imported_setup_dependencies_inner(&import, member, field, ctx);
    }
    if let Some(import) = root_spreads::imported_reexport(program, exported) {
        return imported_setup_dependencies_inner(&import, member, field, ctx);
    }
    for source in root_spreads::star_barrel_sources(program) {
        let import = ImportBinding {
            source: source.to_string(),
            imported: exported.to_string(),
        };
        if let Some(dependencies) = imported_setup_dependencies_inner(&import, member, field, ctx) {
            return Some(dependencies);
        }
    }
    None
}

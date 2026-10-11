use super::{import_bindings, objects, shared, top_level_function_bodies, Ctx, Options};
use crate::codebase::ts_resolver::ImportResolution;
use anyhow::Result;
use oxc_ast::ast::{ObjectExpression, Program};
use std::collections::BTreeSet;
use std::path::Path;

mod deadline_binding;

pub(in crate::integration_tests::test_config::playwright) fn root_options(
    program: &Program<'_>,
    object: &ObjectExpression<'_>,
    source: &str,
    path: &Path,
    resolver: &dyn ImportResolution,
) -> Result<(Options, bool)> {
    root_options_inner(program, object, source, path, resolver)
}

fn root_options_inner(
    program: &Program<'_>,
    object: &ObjectExpression<'_>,
    source: &str,
    path: &Path,
    resolver: &dyn ImportResolution,
) -> Result<(Options, bool)> {
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
    let mut options = objects::project_object_options(object, &mut ctx)?;
    let unproved_binding = !deadline_binding::direct_root(program, &ctx);
    if unproved_binding {
        crate::integration_tests::test_config::deadlines::unproved_case(
            &mut options.deadlines,
            path,
            oxc_span::GetSpan::span(object),
        );
    }
    Ok((options, unproved_binding))
}

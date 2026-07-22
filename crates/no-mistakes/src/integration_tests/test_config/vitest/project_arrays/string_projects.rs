use super::{import_bindings, objects, shared, top_level_function_bodies, Ctx, Options};
use anyhow::Result;
use std::collections::BTreeSet;
use std::path::Path;

/// Vitest allows `test.projects` to name project config files directly. Parse
/// only static strings and feed their exported object through the same
/// object/config extractor as inline projects; never execute the config.
pub(super) fn string_project_options(
    specifier: &str,
    ctx: &mut Ctx<'_, '_>,
) -> Result<Vec<Options>> {
    let Some(path) = ctx.resolver.resolve(specifier, ctx.path) else {
        return Ok(Vec::new());
    };
    if !ctx.seen.insert(path.clone()) {
        return Ok(Vec::new());
    }
    let result = match crate::integration_tests::runner_config::read_request_source(&path) {
        Err(_) => Ok(Vec::new()),
        Ok(source) => crate::integration_tests::runner_config::with_program(
            &path,
            &source,
            |program, source| {
                let bindings = shared::top_level_object_bindings(program);
                let Some(object) = shared::default_export_object(program, &bindings) else {
                    return Ok(Vec::new());
                };
                let mut local_seen = BTreeSet::new();
                let mut object_seen = BTreeSet::new();
                let mut project_ctx = Ctx {
                    source,
                    bindings,
                    functions: top_level_function_bodies(program),
                    imports: import_bindings(program),
                    resolver: ctx.resolver,
                    path: &path,
                    seen: ctx.seen,
                    local_seen: &mut local_seen,
                    object_seen: &mut object_seen,
                };
                let mut options = objects::project_options(object, &mut project_ctx)?;
                options.config_base = path.parent().map(Path::to_path_buf);
                Ok(vec![options])
            },
        )
        .and_then(|options| options),
    };
    ctx.seen.remove(&path);
    result
}

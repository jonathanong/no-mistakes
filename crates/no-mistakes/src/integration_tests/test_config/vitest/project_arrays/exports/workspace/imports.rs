use super::{workspace_exported_options, Ctx, Options};
use crate::integration_tests::test_config::vitest::project_arrays::ImportBinding;
use anyhow::Result;

pub(super) fn imported_workspace_options(
    import: &ImportBinding,
    ctx: &mut Ctx<'_, '_>,
) -> Result<Vec<Options>> {
    let Some(path) = ctx.resolver.resolve(&import.source, ctx.path) else {
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
                let mut local_seen = std::collections::BTreeSet::new();
                let mut object_seen = std::collections::BTreeSet::new();
                let mut nested = Ctx {
                    source,
                    bindings: crate::integration_tests::test_config::vitest::shared::top_level_object_bindings(program),
                    functions: super::super::super::top_level_function_bodies(program),
                    imports: super::super::super::import_bindings(program),
                    resolver: ctx.resolver,
                    path: &path,
                    seen: ctx.seen,
                    local_seen: &mut local_seen,
                    object_seen: &mut object_seen,
                };
                workspace_exported_options(program, &mut nested, &import.imported)
            },
        )?,
    };
    ctx.seen.remove(&path);
    result
}

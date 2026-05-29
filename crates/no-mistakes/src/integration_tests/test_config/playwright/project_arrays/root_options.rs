use super::{import_bindings, objects, shared, top_level_function_bodies, Ctx, Options};
use crate::codebase::ts_resolver::{ImportResolver, TsConfig};
use anyhow::Result;
use oxc_ast::ast::{ObjectExpression, Program};
use std::collections::BTreeSet;
use std::path::Path;

pub(in crate::integration_tests::test_config::playwright) fn root_options(
    program: &Program<'_>,
    object: &ObjectExpression<'_>,
    source: &str,
    path: &Path,
    tsconfig: &TsConfig,
) -> Result<Options> {
    let resolver = ImportResolver::new(tsconfig);
    let mut seen = BTreeSet::new();
    let mut local_seen = BTreeSet::new();
    let mut object_seen = BTreeSet::new();
    let mut ctx = Ctx {
        source,
        bindings: shared::top_level_object_bindings(program),
        functions: top_level_function_bodies(program),
        imports: import_bindings(program),
        resolver: &resolver,
        path,
        seen: &mut seen,
        local_seen: &mut local_seen,
        object_seen: &mut object_seen,
    };
    objects::project_object_options(object, &mut ctx)
}

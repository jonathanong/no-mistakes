use super::{import_bindings, objects, shared, top_level_function_bodies, Ctx, Options};
use crate::codebase::ts_resolver::{ImportResolver, TsConfig};
use anyhow::Result;
use oxc_ast::ast::{ObjectExpression, Program};
use std::collections::{BTreeSet, HashSet};
use std::path::{Path, PathBuf};

pub(in crate::integration_tests::test_config::vitest) fn root_options(
    program: &Program<'_>,
    object: &ObjectExpression<'_>,
    source: &str,
    path: &Path,
    tsconfig: &TsConfig,
) -> Result<Options> {
    root_options_inner(program, object, source, path, tsconfig, None)
}

pub(in crate::integration_tests::test_config::vitest) fn root_options_from_visible(
    program: &Program<'_>,
    object: &ObjectExpression<'_>,
    source: &str,
    path: &Path,
    tsconfig: &TsConfig,
    visible_files: &HashSet<PathBuf>,
) -> Result<Options> {
    root_options_inner(program, object, source, path, tsconfig, Some(visible_files))
}

fn root_options_inner(
    program: &Program<'_>,
    object: &ObjectExpression<'_>,
    source: &str,
    path: &Path,
    tsconfig: &TsConfig,
    visible_files: Option<&HashSet<PathBuf>>,
) -> Result<Options> {
    let resolver = match visible_files {
        Some(visible) => ImportResolver::new(tsconfig).with_visible(visible),
        None => ImportResolver::new(tsconfig),
    };
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
    objects::project_options(object, &mut ctx)
}

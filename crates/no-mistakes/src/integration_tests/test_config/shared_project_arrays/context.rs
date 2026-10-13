use super::{top_level_function_bodies, ImportBinding, ImportContext};
use crate::codebase::ts_resolver::ImportResolution;
use crate::integration_tests::test_config::shared;
use oxc_ast::ast::{Expression, FunctionBody, Program};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub(in crate::integration_tests::test_config) type ExprMap<'a> =
    BTreeMap<String, &'a Expression<'a>>;
pub(in crate::integration_tests::test_config) type FnMap<'a> =
    BTreeMap<String, &'a FunctionBody<'a>>;

/// Both project-array walkers share import and cycle state. The scope flag is
/// meaningful for Vitest nested test objects and always false for Playwright.
pub(in crate::integration_tests::test_config) struct Context<'a, 'r> {
    pub(in crate::integration_tests::test_config) is_test_object: bool,
    pub(in crate::integration_tests::test_config) source: &'a str,
    pub(in crate::integration_tests::test_config) bindings: ExprMap<'a>,
    pub(in crate::integration_tests::test_config) functions: FnMap<'a>,
    pub(in crate::integration_tests::test_config) imports: BTreeMap<String, ImportBinding>,
    pub(in crate::integration_tests::test_config) resolver: &'r dyn ImportResolution,
    pub(in crate::integration_tests::test_config) path: &'r Path,
    pub(in crate::integration_tests::test_config) seen: &'r mut BTreeSet<PathBuf>,
    pub(in crate::integration_tests::test_config) local_seen: &'r mut BTreeSet<String>,
    pub(in crate::integration_tests::test_config) object_seen: &'r mut BTreeSet<String>,
}

impl<'a, 'r> Context<'a, 'r> {
    pub(in crate::integration_tests::test_config) fn for_program(
        program: &'a Program<'a>,
        source: &'a str,
        path: &'r Path,
        resolver: &'r dyn ImportResolution,
        seen: &'r mut BTreeSet<PathBuf>,
        local_seen: &'r mut BTreeSet<String>,
        object_seen: &'r mut BTreeSet<String>,
        imports: BTreeMap<String, ImportBinding>,
        is_test_object: bool,
    ) -> Self {
        Self {
            is_test_object,
            source,
            bindings: shared::top_level_object_bindings(program),
            functions: top_level_function_bodies(program),
            imports,
            resolver,
            path,
            seen,
            local_seen,
            object_seen,
        }
    }
}

impl ImportContext for Context<'_, '_> {
    fn resolver(&self) -> &dyn ImportResolution {
        self.resolver
    }

    fn seen(&mut self) -> &mut BTreeSet<PathBuf> {
        self.seen
    }
}

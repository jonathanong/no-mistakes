use super::super::{CheckFactPlan, PlaywrightFactPlan};
use super::plan::ts_extract_context;
use crate::codebase::rules::test_reliability::{
    self, playwright_provenance::FixtureInitializers, Location,
};
use crate::codebase::ts_source::facts::{self, TsFactPlan, TsFileFacts};
use oxc_ast::ast::{ModuleDeclaration, Program};
use oxc_semantic::{SemanticBuilder, SemanticBuilderReturn};
use std::{path::Path, sync::Arc};

// File-specific annotation candidates extend the request demand before its
// single semantic/fact pass; no competing collector is created.
pub(super) struct Demand<'a> {
    pub request: &'a CheckFactPlan,
    pub ts: TsFactPlan,
}

pub(super) struct PreparedProgramFacts<'a> {
    pub ts: TsFileFacts,
    pub semantic: Option<SemanticBuilderReturn<'a>>,
    pub test_reliability: Vec<Location>,
    pub fixture_initializers: Option<FixtureInitializers>,
}

pub(super) fn collect<'a>(
    root: &Path,
    path: &Path,
    plan: Demand<'_>,
    playwright: Option<&PlaywrightFactPlan>,
    source: &str,
    program: &'a Program<'a>,
    stored_source: Option<Arc<str>>,
) -> PreparedProgramFacts<'a> {
    let reliability_demand = plan.request.test_reliability_paths.get(path);
    let fixture_support = plan.request.playwright_fixture_support
        && program.body.iter().any(|statement| {
            statement
                .as_module_declaration()
                .is_some_and(ModuleDeclaration::is_export)
        });
    let fixture_has_initializers = fixture_support;
    let semantic = (reliability_demand.is_some()
        || fixture_has_initializers
        || playwright.and_then(|plan| plan.file(path)).is_some())
    .then(|| {
        SemanticBuilder::new()
            .with_build_nodes(true)
            .with_check_syntax_error(true)
            .build(program)
    });
    let mut ts_plan = plan.ts;
    ts_plan.imports |= fixture_support;
    let ts = facts::collect_file_facts_from_program_with_semantic(
        path,
        ts_plan,
        &ts_extract_context(root, plan.request),
        source,
        program,
        (None, stored_source),
        semantic.as_ref(),
    );
    let test_reliability =
        reliability_demand
            .zip(semantic.as_ref())
            .map_or_else(Vec::new, |(demand, semantic)| {
                test_reliability::ast::collect(*demand, source, program, &semantic.semantic)
            });
    let fixture_initializers = semantic
        .as_ref()
        .filter(|_| fixture_has_initializers)
        .map(|semantic| {
            test_reliability::ast::fixture_initializers(
                program,
                &semantic.semantic,
                &ts.exported_bindings,
            )
        })
        .filter(|fixtures| {
            !fixtures.values.is_empty()
                || !fixtures.uncertain.is_empty()
                || !fixtures.wrappers.is_empty()
        });
    PreparedProgramFacts {
        ts,
        semantic,
        test_reliability,
        fixture_initializers,
    }
}

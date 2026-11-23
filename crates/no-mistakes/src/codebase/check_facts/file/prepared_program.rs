use super::super::{CheckFactPlan, PlaywrightFactPlan};
use super::plan::ts_extract_context;
use crate::codebase::rules::test_reliability::{self, Location};
use crate::codebase::ts_source::facts::{self, TsFactPlan, TsFileFacts};
use oxc_ast::ast::Program;
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
    let semantic = (reliability_demand.is_some()
        || playwright.and_then(|plan| plan.file(path)).is_some())
    .then(|| {
        SemanticBuilder::new()
            .with_build_nodes(true)
            .with_check_syntax_error(true)
            .build(program)
    });
    let ts = facts::collect_file_facts_from_program_with_semantic(
        path,
        plan.ts,
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
    PreparedProgramFacts {
        ts,
        semantic,
        test_reliability,
    }
}

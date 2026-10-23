use super::super::{CheckFactPlan, PlaywrightFactPlan};
use super::plan::{ts_extract_context, ts_extract_plan};
use crate::codebase::ts_source::facts::{self, TsFileFacts};
use oxc_ast::ast::Program;
use oxc_semantic::{SemanticBuilder, SemanticBuilderReturn};
use std::{path::Path, sync::Arc};

pub(super) struct PreparedProgramFacts<'a> {
    pub ts: TsFileFacts,
    pub semantic: Option<SemanticBuilderReturn<'a>>,
}

pub(super) fn collect<'a>(
    root: &Path,
    path: &Path,
    plan: &CheckFactPlan,
    playwright: Option<&PlaywrightFactPlan>,
    source: &str,
    program: &'a Program<'a>,
    stored_source: Option<Arc<str>>,
) -> PreparedProgramFacts<'a> {
    // One lexical binding projection serves module facts and every Playwright variant.
    let semantic = playwright.and_then(|plan| plan.file(path)).map(|_| {
        SemanticBuilder::new()
            .with_build_nodes(true)
            .with_check_syntax_error(true)
            .build(program)
    });
    let ts = facts::collect_file_facts_from_program_with_semantic(
        path,
        ts_extract_plan(root, plan, path, playwright),
        &ts_extract_context(root, plan),
        source,
        program,
        (None, stored_source),
        semantic.as_ref(),
    );
    PreparedProgramFacts { ts, semantic }
}

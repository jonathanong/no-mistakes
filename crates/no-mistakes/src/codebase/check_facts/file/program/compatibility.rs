use super::*;
use crate::codebase::check_facts::{CheckFactPlan, PlaywrightFactPlan};

pub(crate) fn collect_file_facts_from_program(
    root: &Path,
    path: &Path,
    plan: &CheckFactPlan,
    playwright: Option<&PlaywrightFactPlan>,
    source: &str,
    program: &oxc_ast::ast::Program<'_>,
    owned_source: Option<Arc<str>>,
) -> CheckFileFacts {
    collect_file_facts_from_program_with_discovery(
        &CheckFactVariant {
            root,
            plan,
            playwright,
            collect_tests: true,
        },
        path,
        source,
        program,
        owned_source,
    )
}

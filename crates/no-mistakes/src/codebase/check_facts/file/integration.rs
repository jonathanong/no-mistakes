use super::super::{family_scope::applies, CheckFactPlan};
use crate::integration_tests::types::FileAnalysis;
use oxc_ast::ast::Program;
use std::path::Path;

pub(super) fn collect(
    path: &Path,
    program: &Program<'_>,
    source: &str,
    plan: &CheckFactPlan,
) -> Option<FileAnalysis> {
    (plan.integration && applies(&plan.integration_paths, path)).then(|| {
        crate::integration_tests::analysis::analyze_program_with_test_discovery(
            path,
            program,
            source,
            applies(&plan.integration_test_paths, path),
        )
    })
}

use super::CheckFactPlan;
use crate::codebase::postgres::{sql_identifier_syntax, EmbeddedSqlOptions};
use crate::codebase::ts_source::facts::TsFileFacts;
use oxc_ast::ast::Program;
use std::path::Path;

pub(super) fn selected(root: &Path, path: &Path, plan: &CheckFactPlan) -> bool {
    plan.sql_identifier_profiles
        .values()
        .any(|scans| scans.iter().any(|scan| scan.includes(root, path)))
}
fn profiles(root: &Path, path: &Path, plan: &CheckFactPlan) -> Vec<EmbeddedSqlOptions> {
    plan.sql_identifier_profiles
        .iter()
        .filter(|(_, scans)| scans.iter().any(|scan| scan.includes(root, path)))
        .map(|(profile, _)| profile.clone())
        .collect()
}
pub(super) fn collect(
    root: &Path,
    path: &Path,
    source: &str,
    program: &Program<'_>,
    ts: &TsFileFacts,
    plan: &CheckFactPlan,
) -> sql_identifier_syntax::FileFacts {
    let profiles = profiles(root, path, plan);
    if profiles.is_empty() {
        return Default::default();
    }
    sql_identifier_syntax::collect(program, ts, source, &profiles)
}
pub(super) fn invalid(
    root: &Path,
    path: &Path,
    source: &str,
    program: &Program<'_>,
    plan: &CheckFactPlan,
) -> sql_identifier_syntax::FileFacts {
    sql_identifier_syntax::collect_invalid(program, source, &profiles(root, path, plan))
}

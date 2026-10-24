use super::CheckFactPlan;
use crate::codebase::postgres::{sql_identifier_syntax, EmbeddedSqlOptions};
use crate::codebase::ts_source::facts::TsFileFacts;
use oxc_ast::ast::Program;
use std::path::Path;

#[cfg(test)]
mod tests;

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
/// Return only the configured typed-executor identities whose low-effort scan
/// selects this file, grouped by the exact embedded-SQL profile they support.
pub(super) fn typed_executor_types_by_profile(
    root: &Path,
    path: &Path,
    plan: &CheckFactPlan,
) -> Vec<(
    EmbeddedSqlOptions,
    Vec<crate::config::v2::schema::EffectTargetConfig>,
)> {
    plan.sql_identifier_profiles
        .iter()
        .filter_map(|(profile, scans)| {
            let mut types = scans
                .iter()
                .filter(|scan| scan.includes(root, path))
                .flat_map(|scan| scan.typed_executor_types().iter().cloned())
                .collect::<Vec<_>>();
            if types.is_empty() {
                return None;
            }
            types.sort_by(|left, right| {
                (&left.module, &left.export).cmp(&(&right.module, &right.export))
            });
            types
                .dedup_by(|left, right| left.module == right.module && left.export == right.export);
            Some((profile.clone(), types))
        })
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
    let typed = typed_executor_types_by_profile(root, path, plan)
        .into_iter()
        .collect::<std::collections::BTreeMap<_, _>>();
    if typed.is_empty() {
        sql_identifier_syntax::collect(program, ts, source, &profiles)
    } else {
        sql_identifier_syntax::collect_with_types(program, ts, source, &profiles, &typed)
    }
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

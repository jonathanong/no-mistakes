use super::CheckFactPlan;
use std::path::Path;

#[cfg(not(test))]
macro_rules! record_sql_walk {
    () => {};
}

#[cfg(test)]
#[macro_use]
mod work_probe;

pub(super) fn retains_provider(ts: &crate::codebase::ts_source::facts::TsFileFacts) -> bool {
    ts.operational_error.is_some()
        || ts.parse_error.is_some()
        || !ts.exported_bindings.is_empty()
        || !ts.star_reexport_specifiers.is_empty()
}

pub(super) fn prepared_embedded_sql(
    root: &Path,
    path: &Path,
    source: &str,
    program: &oxc_ast::ast::Program<'_>,
    plan: &CheckFactPlan,
) -> Vec<(
    crate::codebase::postgres::EmbeddedSqlOptions,
    crate::codebase::postgres::EmbeddedSqlFileFacts,
)> {
    // Recovery callers may not have complete export metadata.
    prepare(root, path, source, program, plan, true)
}

pub(super) fn profiles(plan: &CheckFactPlan) -> Vec<crate::codebase::postgres::EmbeddedSqlOptions> {
    let mut profiles = plan.embedded_sql_options.clone();
    if plan.embedded_sql && profiles.is_empty() {
        profiles.push(crate::codebase::postgres::EmbeddedSqlOptions::default());
    }
    profiles.sort();
    profiles.dedup();
    profiles
}

pub(super) fn prepare(
    root: &Path,
    path: &Path,
    source: &str,
    program: &oxc_ast::ast::Program<'_>,
    plan: &CheckFactPlan,
    has_provider_exports: bool,
) -> Vec<(
    crate::codebase::postgres::EmbeddedSqlOptions,
    crate::codebase::postgres::EmbeddedSqlFileFacts,
)> {
    let rel = crate::codebase::ts_source::relative_slash_path(root, path);
    profiles(plan)
        .into_iter()
        // Import projection consumes these same export tables. Keep every
        // potential provider, including unreachable exporters, without a
        // second parse or a separate import graph.
        .filter(|profile| {
            has_provider_exports
                || annotation_needs_profile(plan, profile)
                || plan.scans_embedded_sql(profile, &rel)
        })
        .map(|options| {
            record_sql_walk!();
            let mut facts = crate::codebase::postgres::extract_embedded_sql_from_program(
                path, program, source, &options,
            );
            assign_unproven_rules(
                &mut facts,
                plan.embedded_sql_rule_ids.get(&options).map(Vec::as_slice),
                &rel,
                source,
            );
            (options, facts)
        })
        .collect()
}

fn annotation_needs_profile(
    plan: &CheckFactPlan,
    profile: &crate::codebase::postgres::EmbeddedSqlOptions,
) -> bool {
    // Both annotation projections consume raw call spans in private modules.
    plan.query_annotation_heads
        || plan.query_annotation
            && plan
                .query_annotation_options
                .as_ref()
                .is_none_or(|profiles| profiles.contains(profile))
}

#[cfg(test)]
mod tests;

fn assign_unproven_rules(
    facts: &mut crate::codebase::postgres::EmbeddedSqlFileFacts,
    scans: Option<&[crate::codebase::postgres::EmbeddedSqlRuleScan]>,
    rel: &str,
    source: &str,
) {
    let Some(scans) = scans else {
        crate::codebase::postgres::retain_unproven_spans(facts, source);
        return;
    };
    let mut ids = Vec::new();
    for scan in scans {
        if scan.paths.includes(rel) && !ids.contains(&scan.rule_id) {
            ids.push(scan.rule_id.clone());
        }
    }
    if ids.is_empty() {
        facts.unproven_member_query_spans.clear();
        facts.record_unproven_spans = false;
        return;
    }
    facts.postgres_rule_ids = ids;
    crate::codebase::postgres::retain_unproven_spans(facts, source);
}

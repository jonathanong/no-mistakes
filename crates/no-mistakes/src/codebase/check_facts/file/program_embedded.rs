use super::{CheckFactPlan, Path};

pub(in crate::codebase::check_facts::file) fn prepared_embedded_sql(
    root: &Path,
    path: &Path,
    source: &str,
    program: &oxc_ast::ast::Program<'_>,
    plan: &CheckFactPlan,
) -> Vec<(
    crate::codebase::postgres::EmbeddedSqlOptions,
    crate::codebase::postgres::EmbeddedSqlFileFacts,
)> {
    let mut profiles = plan.embedded_sql_options.clone();
    if plan.embedded_sql && profiles.is_empty() {
        profiles.push(crate::codebase::postgres::EmbeddedSqlOptions::default());
    }
    profiles.sort();
    profiles.dedup();
    let rel = crate::codebase::ts_source::relative_slash_path(root, path);
    profiles
        .into_iter()
        .map(|options| {
            let mut facts =
                crate::codebase::postgres::embedded::extract_embedded_sql_from_program_with_demand(
                    path,
                    program,
                    source,
                    &options,
                    plan.embedded_sql_head_calls_only.contains(&options),
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

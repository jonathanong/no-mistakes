use super::run_rule::RunRuleRequest;
use super::*;

mod deferred;
mod postgres;
mod sourced;

pub(super) fn run(request: RunRuleRequest<'_>) -> Result<Vec<RuleFinding>> {
    let RunRuleRequest {
        rule_id,
        fallback,
        root,
        config,
        files,
        snapshot,
        sources,
        facts,
        defer_suppression,
        session,
        ..
    } = request;
    if rule_id == VITEST_DEFAULT_EXCLUDES {
        return vitest_default_excludes::check_prepared(
            root, config, files, facts, sources, session,
        );
    }
    if rule_id == VITEST_PROJECT_POLICY {
        return vitest_project_policy::check_with_files_and_sources(
            root,
            config,
            &snapshot.paths_for(root),
            sources,
        );
    }
    if rule_id == POSTGRES_EXPLAIN_WRITE_ROLLBACK {
        return postgres_explain_write_rollback::diagnostics_with_files_sources_and_facts(
            root, config, files, sources, facts, false,
        )
        .map(|diagnostics| diagnostics.findings);
    }
    if let Some(out) = postgres::run(rule_id, root, config, files, sources, facts) {
        return out;
    }
    if let Some(out) = deferred::run(rule_id, root, config, files, sources, defer_suppression) {
        return out;
    }
    if let Some(out) = sourced::run(rule_id, root, config, files, sources, defer_suppression) {
        return out;
    }
    fallback(root, config, files)
}

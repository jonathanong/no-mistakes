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
        ..
    } = request;
    if rule_id == VITEST_PROJECT_POLICY {
        return vitest_project_policy::check_with_files_and_sources(
            root,
            config,
            &snapshot.paths_for(root),
            sources,
        );
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

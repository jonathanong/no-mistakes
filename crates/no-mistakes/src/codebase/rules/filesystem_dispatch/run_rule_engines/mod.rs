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
        sources,
        defer_suppression,
        ..
    } = request;
    if let Some(out) = postgres::run(rule_id, root, config, files, sources) {
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

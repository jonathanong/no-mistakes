use super::{Options, RULE_ID};
use crate::codebase::rules::validate_options::checks::{invalid, nonblank, option};
use anyhow::Result;

pub(super) fn selectors(options: &Options) -> Result<()> {
    for (key, jobs) in [
        ("requiredJobs", &options.required_jobs),
        ("forbiddenJobs", &options.forbidden_jobs),
    ] {
        for (index, id) in jobs.iter().enumerate() {
            nonblank(RULE_ID, option(key).index(index), id)?;
        }
    }
    for (index, order) in options.step_orders.iter().enumerate() {
        let path = option("stepOrders").index(index);
        nonblank(RULE_ID, path.clone().key("jobId"), &order.job_id)?;
        for (index, step) in order.steps.iter().enumerate() {
            let path = path.clone().key("steps").index(index);
            let fields = [("id", &step.id), ("uses", &step.uses), ("name", &step.name)];
            if fields.iter().all(|(_, value)| value.is_none()) {
                return Err(invalid(RULE_ID, path, "must specify id, uses, or name"));
            }
            for (key, value) in fields {
                if let Some(value) = value {
                    nonblank(RULE_ID, path.clone().key(key), value)?;
                }
            }
        }
    }
    artifact_edges(options)
}

fn artifact_edges(options: &Options) -> Result<()> {
    for (index, edge) in options.required_artifact_edges.iter().enumerate() {
        let path = option("requiredArtifactEdges").index(index);
        for (key, value) in [("from", &edge.from), ("to", &edge.to), ("name", &edge.name)] {
            nonblank(RULE_ID, path.clone().key(key), value)?;
        }
        if let Some(kind) = edge.match_kind.as_deref() {
            if !matches!(kind, "exact" | "pattern" | "all" | "possible") {
                return Err(invalid(RULE_ID, path.key("match"), "must be exact, pattern, all, or possible"));
            }
        }
    }
    Ok(())
}

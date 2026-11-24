use super::*;

pub(crate) fn queues_configured(config: &NoMistakesConfig) -> bool {
    config.queues.unbounded_job_retention.is_some()
        || config.queues.job_state_as_record.is_some()
        || queue_topology_configured(config)
}

pub(super) fn queue_topology_configured(config: &NoMistakesConfig) -> bool {
    config
        .projects
        .values()
        .any(|project| !project.queues.enqueues.is_empty() || !project.queues.workers.is_empty())
}

pub(crate) fn unique_exports_configured(config: &NoMistakesConfig) -> bool {
    rule_configured(config, unique_exports::RULE_ID)
}

pub(crate) fn rule_configured(config: &NoMistakesConfig, rule_id: &str) -> bool {
    config.rule_configured(rule_id)
}

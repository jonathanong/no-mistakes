use crate::check_parallel::DomainResults;
use crate::check_tasks::CheckTask;
use anyhow::anyhow;
use no_mistakes::codebase::rules::RuleFinding;
use no_mistakes::codebase::unique_exports::PreparedUniqueExportFinding;
use no_mistakes::integration_tests::IntegrationFinding;
use no_mistakes::queue::CheckFinding;
use no_mistakes::react_traits;
use std::time::Duration;

fn empty_task<T>(findings: T) -> CheckTask<T> {
    CheckTask {
        findings,
        react_suppression_targets: Vec::new(),
        suppression_sources: Vec::new(),
        advisories: Vec::new(),
        warning: None,
        duration: Duration::ZERO,
    }
}

fn ok_react() -> anyhow::Result<CheckTask<Vec<react_traits::Violation>>> {
    Ok(empty_task(Vec::new()))
}

fn ok_queues() -> anyhow::Result<CheckTask<Vec<CheckFinding>>> {
    Ok(empty_task(Vec::new()))
}

fn ok_rules() -> anyhow::Result<CheckTask<Vec<RuleFinding>>> {
    Ok(empty_task(Vec::new()))
}

fn ok_integration() -> anyhow::Result<CheckTask<Vec<IntegrationFinding>>> {
    Ok(empty_task(Vec::new()))
}

fn ok_codebase() -> anyhow::Result<CheckTask<Vec<PreparedUniqueExportFinding>>> {
    Ok(empty_task(Vec::new()))
}

pub(super) fn err_react() -> DomainResults {
    (
        Err(anyhow!("react")),
        ok_queues(),
        ok_rules(),
        ok_integration(),
        ok_codebase(),
        ok_rules(),
    )
}

pub(super) fn err_queues() -> DomainResults {
    (
        ok_react(),
        Err(anyhow!("queues")),
        ok_rules(),
        ok_integration(),
        ok_codebase(),
        ok_rules(),
    )
}

pub(super) fn err_rules() -> DomainResults {
    (
        ok_react(),
        ok_queues(),
        Err(anyhow!("rules")),
        ok_integration(),
        ok_codebase(),
        ok_rules(),
    )
}

pub(super) fn err_integration() -> DomainResults {
    (
        ok_react(),
        ok_queues(),
        ok_rules(),
        Err(anyhow!("integration")),
        ok_codebase(),
        ok_rules(),
    )
}

pub(super) fn err_codebase() -> DomainResults {
    (
        ok_react(),
        ok_queues(),
        ok_rules(),
        ok_integration(),
        Err(anyhow!("codebase")),
        ok_rules(),
    )
}

pub(super) fn err_filesystem_rules() -> DomainResults {
    (
        ok_react(),
        ok_queues(),
        ok_rules(),
        ok_integration(),
        ok_codebase(),
        Err(anyhow!("filesystem_rules")),
    )
}

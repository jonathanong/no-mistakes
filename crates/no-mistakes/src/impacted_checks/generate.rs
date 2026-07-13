//! Report generation for `impacted-checks`: reuse one prepared test-plan
//! request across frameworks and apply the configured generic checks.

use super::{CheckCommand, CheckKind, ImpactedChecksArgs, ImpactedChecksReport};
use crate::tests::Warning;
use anyhow::Result;

mod args;
mod generic;
mod prepare;
pub(super) use args::plan_args_for;
use args::{discover_phase, select_phase};
pub(super) use generic::{dedupe_checks, dedupe_warnings, generic_checks};
use prepare::prepare_impacted_checks;

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct PlanStats {
    pub(crate) framework_discoveries: usize,
    pub(crate) graph_builds: usize,
}

/// Compute the impacted-checks report (shared by the CLI and N-API).
pub fn generate_impacted_checks(args: &ImpactedChecksArgs) -> Result<ImpactedChecksReport> {
    Ok(generate_impacted_checks_with_stats(args)?.0)
}

pub(crate) fn generate_impacted_checks_with_stats(
    args: &ImpactedChecksArgs,
) -> Result<(ImpactedChecksReport, PlanStats)> {
    let mut timing = super::timing::TimingTracker::new(false, false);
    generate_impacted_checks_with_timing(args, &mut timing)
}

pub(crate) fn generate_impacted_checks_with_timing(
    args: &ImpactedChecksArgs,
    timing: &mut super::timing::TimingTracker,
) -> Result<(ImpactedChecksReport, PlanStats)> {
    let prepared = timing.run_phase("prepare", || prepare_impacted_checks(args))?;
    let frameworks = prepared.frameworks.clone();

    let mut checks: Vec<CheckCommand> = Vec::new();
    let mut warnings: Vec<Warning> = Vec::new();
    let mut fallback_triggered = false;
    let graph_builds;

    if let Some(request) = prepared.framework_request() {
        for framework in &frameworks {
            timing.run_phase(discover_phase(*framework), || {
                request.discover_tests(*framework).map(|_| ())
            })?;
        }

        for framework in &frameworks {
            let framework_args = plan_args_for(args, Some(*framework));
            let phase = select_phase(*framework);
            let started = timing.start_phase(phase);
            let plan_result = crate::tests::plan::generate_plan_with_prepared(
                &framework_args,
                request,
                Some(timing),
            );
            let plan = match plan_result {
                Ok(plan) => {
                    timing.finish_phase(phase, started);
                    plan
                }
                Err(error) => {
                    timing.fail_phase(phase, started);
                    return Err(error);
                }
            };
            fallback_triggered |= plan.fallback_triggered;
            warnings.extend(plan.warnings.iter().cloned());
            append_test_checks(&mut checks, &plan);
        }
        graph_builds = usize::from(request.graph_is_initialized());
    } else {
        graph_builds = 0;
    }

    let generic = timing.run_phase("generic-checks", || {
        generic_checks(
            prepared.config(),
            &prepared.changed_files,
            &prepared.missing,
        )
    })?;
    checks.extend(generic);

    let stats = prepared.stats();
    debug_assert_eq!(stats.graph_builds, graph_builds);
    let report = ImpactedChecksReport {
        changed_files: prepared.changed_files,
        checks: dedupe_checks(checks),
        warnings: dedupe_warnings(warnings),
        fallback_triggered,
    };
    Ok((report, stats))
}

fn append_test_checks(checks: &mut Vec<CheckCommand>, plan: &crate::tests::TestPlan) {
    for test in &plan.selected_tests {
        for target in &test.targets {
            let mut command = target.base_command.clone();
            command.extend(target.runner_args.iter().cloned());
            checks.push(CheckCommand {
                name: target.runner.clone(),
                kind: CheckKind::Test,
                command,
                files: vec![test.test_file.clone()],
            });
        }
    }
}

use super::super::ScanContext;
use crate::codebase::ci_graph::triggers::{CompiledTriggers, TriggerMatch};
use crate::codebase::rules::tsconfig_gate_coverage::{
    application::resolve_gate_project_against_tracked, command_scan,
};
use std::collections::BTreeSet;

pub(super) fn scan(
    run: &str,
    cwd: &str,
    failure_enforced: bool,
    triggers: &CompiledTriggers,
    context: &ScanContext<'_>,
) -> BTreeSet<String> {
    let scanned = if failure_enforced {
        command_scan::scan_shell_for_typechecked_projects(run, cwd)
    } else {
        command_scan::scan_workflow_shell_for_typechecked_projects(run, cwd, false)
    };
    scanned
        .into_iter()
        .map(|project| resolve_gate_project_against_tracked(&project, context.tracked))
        .filter(|project| {
            context
                .project_source_inputs
                .get(project)
                .is_some_and(|source_inputs| {
                    source_inputs.iter().all(|input| {
                        matches!(
                            triggers.evaluate(input).0,
                            TriggerMatch::Matched | TriggerMatch::Always
                        )
                    })
                })
        })
        .collect()
}

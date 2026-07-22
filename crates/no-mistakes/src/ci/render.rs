//! Output rendering for the `ci` commands.

use super::{CiEnvReport, CiImpactReport, EnvLocationKind, EnvScope, Format, TopologyFormat};
use crate::codebase::ci_graph::model::PermissionLevel;
use crate::codebase::ci_graph::permissions::ResolvedPermissions;
use crate::codebase::ci_graph::triggers::TriggerMatch;
use crate::codebase::workflow_topology::model::{WorkflowTopology, WorkflowTopologyDiagnostic};
use crate::codebase::workflow_topology::{render_json, render_mermaid};
use anyhow::Result;

pub(super) fn render_impact(report: &CiImpactReport, format: Format) -> Result<String> {
    Ok(match format {
        Format::Json => format!("{}\n", serde_json::to_string_pretty(report)?),
        Format::Yml => serde_yaml::to_string(report)?,
        Format::Paths => report
            .workflows
            .iter()
            .map(|workflow| format!("{}\n", workflow.path))
            .collect(),
        Format::Md => render_impact_text(report, "- "),
        Format::Human => render_impact_text(report, ""),
    })
}

fn render_impact_text(report: &CiImpactReport, bullet: &str) -> String {
    let mut out = String::new();
    if report.workflows.is_empty() {
        out.push_str("No workflows triggered.\n");
    }
    for workflow in &report.workflows {
        let marker = match workflow.trigger {
            TriggerMatch::Always => " (always)",
            _ => "",
        };
        let reusable = if workflow.reusable { " [reusable]" } else { "" };
        out.push_str(&format!("{bullet}{}{marker}{reusable}\n", workflow.path));
        for job in &workflow.jobs {
            out.push_str(&format!(
                "  {bullet}{}: {}\n",
                job.id,
                format_permissions(&job.permissions)
            ));
        }
    }
    for warning in &report.warnings {
        out.push_str(&format!("warning: {}: {}\n", warning.path, warning.message));
    }
    out
}

fn format_permissions(permissions: &ResolvedPermissions) -> String {
    let scopes: Vec<String> = permissions
        .scopes
        .iter()
        .map(|(scope, level)| format!("{scope}:{}", level_str(*level)))
        .collect();
    let body = if scopes.is_empty() {
        "(none)".to_string()
    } else {
        scopes.join(", ")
    };
    if permissions.assumed_default {
        format!("{body} (assumed default)")
    } else {
        body
    }
}

fn level_str(level: PermissionLevel) -> &'static str {
    match level {
        PermissionLevel::Read => "read",
        PermissionLevel::Write => "write",
        PermissionLevel::None => "none",
    }
}

pub(super) fn render_env(report: &CiEnvReport, format: Format) -> Result<String> {
    Ok(match format {
        Format::Json => format!("{}\n", serde_json::to_string_pretty(report)?),
        Format::Yml => serde_yaml::to_string(report)?,
        Format::Paths => report
            .files
            .iter()
            .map(|file| format!("{}\n", file.path))
            .collect(),
        Format::Md => render_env_text(report, "- "),
        Format::Human => render_env_text(report, ""),
    })
}

fn render_env_text(report: &CiEnvReport, bullet: &str) -> String {
    let mut out = String::new();
    if report.files.is_empty() {
        out.push_str(&format!(
            "`{}` not found in any workflow.\n",
            report.variable
        ));
    }
    for file in &report.files {
        out.push_str(&format!("{bullet}{}\n", file.path));
        for location in &file.locations {
            let kind = match location.kind {
                EnvLocationKind::Definition => "definition",
                EnvLocationKind::Reference => "reference",
            };
            let scope = match location.scope {
                EnvScope::Workflow => "workflow",
                EnvScope::Job => "job",
                EnvScope::Step => "step",
            };
            let job = location
                .job
                .as_ref()
                .map(|job| format!(" job={job}"))
                .unwrap_or_default();
            let value = location
                .value
                .as_ref()
                .map(|value| format!(" = {value}"))
                .unwrap_or_default();
            out.push_str(&format!("  {bullet}{kind} @ {scope}{job}{value}\n"));
        }
    }
    for warning in &report.warnings {
        out.push_str(&format!("warning: {}: {}\n", warning.path, warning.message));
    }
    out
}

pub(super) fn render_topology(report: &WorkflowTopology, format: TopologyFormat) -> Result<String> {
    Ok(match format {
        TopologyFormat::Json => render_json::render_workflow_topology_json(report)?,
        TopologyFormat::Mermaid => render_mermaid::render_workflow_topology_mermaid(report),
    })
}

/// `[<code>] <workflowPath>(<space><jobId in parens>)?: <message>`, matching
/// the original engine's CLI error formatting for each diagnostic line.
pub(super) fn format_topology_diagnostic(diagnostic: &WorkflowTopologyDiagnostic) -> String {
    let job_suffix = diagnostic
        .job_id
        .as_deref()
        .map(|id| format!(" ({id})"))
        .unwrap_or_default();
    format!(
        "[{}] {}{job_suffix}: {}",
        diagnostic.code.as_str(),
        diagnostic.workflow_path,
        diagnostic.message
    )
}

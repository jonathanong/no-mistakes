use super::findings::{inherited, unresolved};
use super::permissions;
use crate::codebase::rules::{self, analysis_policy::AnalysisDiagnostics};
use crate::config::v2::Unanalyzable;
use serde_yaml::Value;

pub(super) fn check_source(rel: &str, source: &str, policy: Unanalyzable) -> AnalysisDiagnostics {
    let mut diagnostics = AnalysisDiagnostics::default();
    let locations = rules::github_actions_workflow_files::job_lines::collect(source);
    let value: Value = match serde_yaml::from_str(source) {
        Ok(value) => value,
        Err(error) => {
            diagnostics.unanalyzable(
                policy,
                unresolved(
                    rel,
                    error.location().map_or(1, |location| location.line()),
                    "unanalyzable",
                    &format!("invalid YAML ({error})"),
                ),
            );
            return diagnostics;
        }
    };
    let Some(root) = value.as_mapping() else {
        diagnostics.unanalyzable(
            policy,
            unresolved(rel, 1, "unanalyzable", "workflow is not a mapping"),
        );
        return diagnostics;
    };
    let root_permission = root.get(key("permissions"));
    let summary = root_permission
        .map(permissions::inspect)
        .unwrap_or_default();
    if summary.unresolved || root.contains_key(key("<<")) {
        diagnostics.unanalyzable(
            policy,
            unresolved(
                rel,
                locations.permissions_line(),
                "unanalyzable",
                "workflow permissions are not fully static",
            ),
        );
    }
    let Some(jobs_value) = root.get(key("jobs")) else {
        diagnostics.unanalyzable(
            policy,
            unresolved(
                rel,
                locations.jobs_line(),
                "unanalyzable",
                "workflow has no jobs to analyze",
            ),
        );
        return diagnostics;
    };
    let Some(jobs) = jobs_value.as_mapping() else {
        diagnostics.unanalyzable(
            policy,
            unresolved(
                rel,
                locations.jobs_line(),
                "unanalyzable",
                "jobs is not a mapping",
            ),
        );
        return diagnostics;
    };
    if jobs.is_empty() {
        diagnostics.unanalyzable(
            policy,
            unresolved(
                rel,
                locations.jobs_line(),
                "unanalyzable",
                "workflow has no jobs to analyze",
            ),
        );
    }
    for (job_key, job_value) in jobs {
        if job_key.as_str() == Some("<<") {
            diagnostics.unanalyzable(
                policy,
                unresolved(
                    rel,
                    locations.jobs_line(),
                    "unanalyzable",
                    "jobs merge may supply additional jobs",
                ),
            );
            continue;
        }
        let Some(job_id) = job_key.as_str() else {
            diagnostics.unanalyzable(
                policy,
                unresolved(
                    rel,
                    locations.jobs_line(),
                    "unanalyzable",
                    "job name is not a string",
                ),
            );
            continue;
        };
        let line = locations.line(job_id);
        let Some(job) = job_value.as_mapping() else {
            diagnostics.unanalyzable(
                policy,
                unresolved(rel, line, job_id, "job definition is not a mapping"),
            );
            continue;
        };
        if let Some(own) = job.get(key("permissions")) {
            if permissions::inspect(own).unresolved {
                diagnostics.unanalyzable(
                    policy,
                    unresolved(
                        rel,
                        locations.job_permissions_line(job_id),
                        job_id,
                        "job permissions are not fully static",
                    ),
                );
            }
            continue;
        }
        if job.contains_key(key("<<")) {
            diagnostics.unanalyzable(
                policy,
                unresolved(rel, line, job_id, "job merge may supply permissions"),
            );
            continue;
        }
        if !summary.writes.is_empty() {
            diagnostics
                .findings
                .push(inherited(rel, line, job_id, &summary.writes));
        }
    }
    diagnostics
}

fn key(name: &str) -> Value {
    Value::String(name.to_owned())
}

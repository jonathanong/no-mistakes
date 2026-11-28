use super::diagnostics::RunCtx;
use super::emit::finding;
use crate::codebase::rules::analysis_policy::AnalysisDiagnostics;
use serde_yaml::{Mapping, Value};

pub(super) fn report_local_workflow_call(
    ctx: &RunCtx<'_>,
    rel: &str,
    source: &str,
    job_id: &str,
    job: &Mapping,
    diag: &mut AnalysisDiagnostics,
) {
    let Some(uses) = job
        .get(Value::String("uses".to_owned()))
        .and_then(Value::as_str)
    else {
        return;
    };
    if !uses.trim().starts_with("./") {
        return;
    }
    let line = job_uses_line(source, job_id);
    diag.unanalyzable(
        ctx.unanalyzable,
        finding(
            rel,
            line,
            format!("{rel}:{line}: job `{job_id}` calls local reusable workflow `{uses}`, whose scripts cannot be checked against this workflow's path filters; inline its steps or list its scripts in the path filters."),
            "unanalyzable",
        ),
    );
}

pub(super) fn report_local_uses_outside_repo(
    ctx: &RunCtx<'_>,
    rel: &str,
    line: usize,
    place: &str,
    uses: &str,
    diag: &mut AnalysisDiagnostics,
) {
    diag.unanalyzable(
        ctx.unanalyzable,
        finding(
            rel,
            line,
            format!("{rel}:{line}: {place} uses local action `{uses}`, which leaves the repository and cannot be statically checked against workflow path filters."),
            "unanalyzable",
        ),
    );
}

fn job_uses_line(source: &str, job_id: &str) -> usize {
    let keys = [
        format!("{job_id}:"),
        format!("'{job_id}':"),
        format!("\"{job_id}\":"),
    ];
    let mut in_job = false;
    for (index, line) in source.lines().enumerate() {
        let trimmed = line.trim_start();
        if in_job {
            if trimmed.starts_with("uses:") {
                return index + 1;
            }
        } else {
            in_job = keys.iter().any(|key| trimmed.starts_with(key.as_str()));
        }
    }
    1
}

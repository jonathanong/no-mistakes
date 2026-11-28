use super::composites;
use super::diagnostics::RunCtx;
use super::emit::{emit_hits, finding, EmitCtx, Origin};
use super::scripts::{self, WorkingDir};
use super::triggers;
use crate::codebase::rules::{
    analysis_policy::AnalysisDiagnostics,
    github_actions_script_expressions::{step_run_lines, StepRun},
    github_actions_workflow_files::{run_scripts::scalar_name, step_lines},
};
use serde_yaml::{Mapping, Value};
use std::path::Path;

pub(super) fn check_source(
    ctx: &RunCtx<'_>,
    path: &Path,
    rel: &str,
    source: &str,
) -> AnalysisDiagnostics {
    let mut diag = AnalysisDiagnostics::default();
    let trimmed = source.trim_start_matches('\u{feff}');
    let value = match ctx.sources.parse_loaded_yaml(path, source, true) {
        Ok(value) => value,
        Err(error) => {
            let line = error.line();
            diag.unanalyzable(
                ctx.unanalyzable,
                finding(
                    rel,
                    line,
                    format!("{rel}:{line}: invalid workflow YAML ({error}); fix the YAML so workflow path filters can be checked."),
                    "unanalyzable",
                ),
            );
            return diag;
        }
    };
    let parsed = triggers::parse_triggers(trimmed, &value, &ctx.compiled.events);
    for issue in parsed.unanalyzable {
        diag.unanalyzable(
            ctx.unanalyzable,
            finding(
                rel,
                issue.line,
                format!("{rel}:{}: {}", issue.line, issue.detail),
                "unanalyzable",
            ),
        );
    }
    if parsed.filters.is_empty() {
        return diag;
    }
    let Some(jobs) = value.get("jobs").and_then(Value::as_mapping) else {
        return diag;
    };
    let emit_ctx = EmitCtx {
        rel,
        source: trimmed,
        filters: &parsed.filters,
        unanalyzable: ctx.unanalyzable,
        rule_message: ctx.rule_message,
    };
    let wf_dir = defaults_working_dir(&value, WorkingDir::Known(String::new()));
    let located = step_run_lines(trimmed, false);
    for (id, job) in jobs {
        let Some(job_map) = job.as_mapping() else {
            continue;
        };
        let job_id = scalar_name(id);
        let job_dir = defaults_working_dir(job, wf_dir.clone());
        let Some(steps) = mapping_get(job_map, "steps").and_then(Value::as_sequence) else {
            continue;
        };
        let job_ctx = JobStepCtx {
            run: ctx,
            emit: &emit_ctx,
            located: &located,
            job_id: &job_id,
            job_dir: &job_dir,
        };
        for (index, step) in steps.iter().enumerate() {
            if let Some(step_map) = step.as_mapping() {
                check_step(&job_ctx, index, step_map, &mut diag);
            }
        }
    }
    diag
}

struct JobStepCtx<'a> {
    run: &'a RunCtx<'a>,
    emit: &'a EmitCtx<'a>,
    located: &'a [StepRun],
    job_id: &'a str,
    job_dir: &'a WorkingDir,
}

fn check_step(
    ctx: &JobStepCtx<'_>,
    index: usize,
    step_map: &Mapping,
    diag: &mut AnalysisDiagnostics,
) {
    let step_no = index + 1;
    let place = format!("job `{}` step {step_no}", ctx.job_id);
    if let Some(run_val) = mapping_get(step_map, "run") {
        if let Some(script) = run_val.as_str() {
            let step_dir = step_working_dir(step_map, ctx.job_dir.clone());
            let step_lines = ctx
                .located
                .iter()
                .find(|item| item.job == ctx.job_id && item.step == step_no);
            let hits = scripts::scan_run(
                script,
                mapping_get(step_map, "shell").and_then(Value::as_str),
                step_dir,
                &ctx.run.compiled.script_dirs,
            );
            emit_hits(
                ctx.emit,
                Origin::Run {
                    place: &place,
                    located: step_lines,
                    script,
                },
                &hits,
                diag,
            );
        } else {
            let line = step_lines::first_key_line(ctx.emit.source, ctx.job_id, index, &["run"]);
            diag.unanalyzable(
                ctx.run.unanalyzable,
                finding(ctx.emit.rel, line, format!("{}:{line}: {place} has a non-string `run` value; write a literal shell script so workflow path filters can be checked.", ctx.emit.rel), "unanalyzable"),
            );
        }
    }
    if let Some(uses) = mapping_get(step_map, "uses").and_then(Value::as_str) {
        let uses_line = step_lines::first_key_line(ctx.emit.source, ctx.job_id, index, &["uses"]);
        if uses.trim().starts_with("./") && uses.contains("${{") {
            diag.unanalyzable(
                ctx.run.unanalyzable,
                finding(ctx.emit.rel, uses_line, format!("{}:{uses_line}: {place} uses dynamic local action `{uses}`, which cannot be statically checked against workflow path filters.", ctx.emit.rel), "unanalyzable"),
            );
        } else if let Some(outcome) = composites::scan_local_action(
            ctx.run.root,
            ctx.run.sources,
            uses,
            &ctx.run.compiled.script_dirs,
        ) {
            emit_hits(
                ctx.emit,
                Origin::Composite {
                    place: &place,
                    action: &outcome.action_path,
                    uses_line,
                },
                &outcome.hits,
                diag,
            );
        }
    }
}

fn defaults_working_dir(node: &Value, inherited: WorkingDir) -> WorkingDir {
    match node
        .get("defaults")
        .and_then(|d| d.get("run"))
        .and_then(|r| r.get("working-directory"))
    {
        Some(wd) => parse_working_dir(wd),
        None => inherited,
    }
}

fn step_working_dir(step: &Mapping, inherited: WorkingDir) -> WorkingDir {
    match mapping_get(step, "working-directory") {
        None | Some(Value::Null) => inherited,
        Some(wd) => parse_working_dir(wd),
    }
}

pub(super) fn parse_working_dir(value: &Value) -> WorkingDir {
    match value {
        Value::Null => WorkingDir::Known(String::new()),
        Value::String(raw) => {
            let trimmed = raw.trim();
            if trimmed.contains("${{") || trimmed.contains('$') {
                WorkingDir::Dynamic(trimmed.to_string())
            } else {
                scripts::resolve_rel_path("", trimmed)
                    .map_or(WorkingDir::OutsideRepo, WorkingDir::Known)
            }
        }
        _ => WorkingDir::Dynamic("non-string".to_string()),
    }
}

fn mapping_get<'a>(map: &'a Mapping, key: &str) -> Option<&'a Value> {
    map.get(Value::String(key.to_owned()))
}

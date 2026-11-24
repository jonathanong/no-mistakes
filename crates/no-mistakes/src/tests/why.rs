use crate::tests::plan::generate_plan;
use crate::tests::{ImpactEdgeDetail, TestPlan};
use crate::tests::{PlanArgs, WhyArgs, WhyFormat};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt::Write;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct WhyStep {
    pub node: String,
    pub via: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<ImpactEdgeDetail>,
}

type WhyStepsByChangedFile = BTreeMap<String, Vec<WhyStep>>;

pub(crate) fn run(args: WhyArgs) -> Result<ExitCode> {
    let context = why_context(&args)?;
    let path_steps = why_steps_with_context(&args, &context)?;
    let mut output = String::new();

    if path_steps.is_empty() {
        writeln!(
            output,
            "No path found connecting changed files to test target `{}`.",
            context.test_rel
        )?;
    } else if matches!(args.format, WhyFormat::Text) {
        for (changed_file, steps) in &path_steps {
            writeln!(
                output,
                "Path from `{}` to `{}`:",
                changed_file, context.test_rel
            )?;
            let chain: Vec<String> = steps
                .iter()
                .map(|step| {
                    if let Some(ref via) = step.via {
                        format!(
                            "`{}` ➔ [{}]",
                            step.node,
                            display_via(via, step.detail.as_ref())
                        )
                    } else {
                        format!("`{}`", step.node)
                    }
                })
                .collect();
            writeln!(output, "  {}\n", chain.join(" ➔ "))?;
        }
    } else {
        writeln!(output, "{}", serde_json::to_string(&path_steps)?)?;
    }

    crate::invocation::commit_timeout()?;
    print!("{output}");
    Ok(ExitCode::SUCCESS)
}

fn display_via(via: &str, detail: Option<&ImpactEdgeDetail>) -> String {
    match detail {
        Some(ImpactEdgeDetail::VitestSetup { field }) => format!("{via} ({field})"),
        Some(ImpactEdgeDetail::Resource { .. }) | None => via.to_string(),
    }
}

const _: fn(WhyArgs) -> Result<ExitCode> = run;

pub(crate) fn why_steps(args: &WhyArgs) -> Result<WhyStepsByChangedFile> {
    let context = why_context(args)?;
    why_steps_with_context(args, &context)
}

const _: fn(&WhyArgs) -> Result<WhyStepsByChangedFile> = why_steps;

struct WhyContext {
    root: std::path::PathBuf,
    test_rel: String,
}

fn why_context(args: &WhyArgs) -> Result<WhyContext> {
    let cwd = std::env::current_dir().context("cwd must be accessible")?;
    let root = no_mistakes::cli::resolve_optional_root(Some(&args.root), &cwd);
    let root = no_mistakes::codebase::ts_resolver::normalize_path(&root);
    let test_rel = relative_path_str(&root, &args.test);
    Ok(WhyContext { root, test_rel })
}

fn why_steps_with_context(args: &WhyArgs, context: &WhyContext) -> Result<WhyStepsByChangedFile> {
    // 1. If plan JSON is provided, read from it
    let path_steps = if let Some(ref plan_path) = args.plan {
        read_from_plan(
            plan_path,
            &context.test_rel,
            args.changed
                .as_ref()
                .map(|p| relative_path_str(&context.root, p))
                .as_deref(),
        )?
    } else {
        // 2. Otherwise run live analysis
        run_live_analysis(args, &context.root, &context.test_rel)?
    };

    Ok(path_steps)
}

fn read_from_plan(
    plan_path: &Path,
    test_rel: &str,
    changed_rel: Option<&str>,
) -> Result<BTreeMap<String, Vec<WhyStep>>> {
    let content = fs::read_to_string(plan_path)
        .with_context(|| format!("Failed to read plan from {}", plan_path.display()))?;
    let plan: TestPlan = serde_json::from_str(&content)?;

    Ok(steps_from_plan(&plan, test_rel, changed_rel))
}

/// Convert a decoded plan into `tests why` steps. Keeping this pure lets
/// provenance tests use saved fixtures instead of constructing files at run
/// time, and keeps plan-backed output identical to live analysis.
fn steps_from_plan(
    plan: &TestPlan,
    test_rel: &str,
    changed_rel: Option<&str>,
) -> BTreeMap<String, Vec<WhyStep>> {
    let mut result = BTreeMap::new();

    if let Some(selected) = plan.selected_tests.iter().find(|t| t.test_file == test_rel) {
        for reason in &selected.reasons {
            if let Some(changed) = changed_rel {
                if reason.changed_file != changed {
                    continue;
                }
            }

            let mut steps = Vec::new();
            for i in 0..reason.path.len() {
                let node = reason.path[i].clone();
                let via = if i < reason.via.len() {
                    Some(reason.via[i].clone())
                } else {
                    None
                };
                let detail = reason.via_details.get(i).cloned().flatten();
                steps.push(WhyStep { node, via, detail });
            }
            result.insert(reason.changed_file.clone(), steps);
        }
    }

    result
}

include!("why_live.rs");

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
#[path = "why_tests.rs"]
mod tests;

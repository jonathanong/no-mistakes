use crate::tests::plan::generate_plan;
use crate::tests::TestPlan;
use crate::tests::{PlanArgs, WhyArgs, WhyFormat};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct WhyStep {
    pub node: String,
    pub via: Option<String>,
}

type WhyStepsByChangedFile = BTreeMap<String, Vec<WhyStep>>;

pub(crate) fn run(args: WhyArgs) -> Result<ExitCode> {
    let context = why_context(&args)?;
    let path_steps = why_steps_with_context(&args, &context)?;

    if path_steps.is_empty() {
        println!(
            "No path found connecting changed files to test target `{}`.",
            context.test_rel
        );
        return Ok(ExitCode::SUCCESS);
    }

    match args.format {
        WhyFormat::Text => {
            for (changed_file, steps) in &path_steps {
                println!("Path from `{}` to `{}`:", changed_file, context.test_rel);
                let chain: Vec<String> = steps
                    .iter()
                    .map(|step| {
                        if let Some(ref via) = step.via {
                            format!("`{}` ➔ [{}]", step.node, via)
                        } else {
                            format!("`{}`", step.node)
                        }
                    })
                    .collect();
                println!("  {}\n", chain.join(" ➔ "));
            }
        }
        WhyFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&path_steps)?);
        }
    }

    Ok(ExitCode::SUCCESS)
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
                steps.push(WhyStep { node, via });
            }
            result.insert(reason.changed_file.clone(), steps);
        }
    }

    Ok(result)
}

fn run_live_analysis(
    args: &WhyArgs,
    root: &Path,
    test_rel: &str,
) -> Result<BTreeMap<String, Vec<WhyStep>>> {
    // Generate plan live to find all connections and warn/fallback correctly
    let plan_args = PlanArgs {
        framework: None,
        root: root.to_path_buf(),
        config: args.config.clone(),
        tsconfig: args.tsconfig.clone(),
        base: None,
        head: None,
        changed_file: args.changed.clone().into_iter().collect(),
        changed_files: None,
        diff: None,
        diff_stdin: false,
        diff_command: None,
        entrypoints: Vec::new(),
        diff_content: None,
        environment: "pre-push".to_string(),
        limit_percent: None,
        limit_files: None,
        global_config_fallback: None,
        format: None,
        json: true,
    };

    let plan = generate_plan(&plan_args)?;
    let mut result = BTreeMap::new();

    if let Some(selected) = plan.selected_tests.iter().find(|t| t.test_file == test_rel) {
        for reason in &selected.reasons {
            let mut steps = Vec::new();
            for i in 0..reason.path.len() {
                let node = reason.path[i].clone();
                let via = if i < reason.via.len() {
                    Some(reason.via[i].clone())
                } else {
                    None
                };
                steps.push(WhyStep { node, via });
            }
            result.insert(reason.changed_file.clone(), steps);
        }
    }

    Ok(result)
}

fn relative_path_str(root: &Path, path: &Path) -> String {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    };
    let absolute_normalized = no_mistakes::codebase::ts_resolver::normalize_path(&absolute);
    no_mistakes::codebase::ts_source::relative_slash_path(root, &absolute_normalized)
}

pub(crate) fn resolve_tsconfig(
    tsconfig_arg: Option<&Path>,
    root: &Path,
) -> Result<no_mistakes::codebase::ts_resolver::TsConfig> {
    match tsconfig_arg {
        Some(path) => no_mistakes::codebase::ts_resolver::load_tsconfig(path),
        None => match no_mistakes::codebase::ts_resolver::find_tsconfig(root) {
            Some(path) => no_mistakes::codebase::ts_resolver::load_tsconfig(&path),
            None => Ok(no_mistakes::codebase::ts_resolver::TsConfig {
                dir: root.to_path_buf(),
                paths: vec![],
                paths_dir: root.to_path_buf(),
                base_url: None,
            }),
        },
    }
}

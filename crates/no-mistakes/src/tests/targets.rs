use crate::tests::{TargetsArgs, TargetsFormat, TestFramework, TestPlan};
use anyhow::{bail, Context, Result};
use no_mistakes::codebase::test_discovery::{discover_tests, TestExecutionTarget, TestRunner};
use no_mistakes::config::v2::load_v2_config;
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TestsTargetsReport {
    pub framework: String,
    pub tests: Vec<TestTargetRow>,
    pub warnings: Vec<TargetWarning>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TestTargetRow {
    pub test_file: String,
    pub targets: Vec<TestExecutionTarget>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TargetWarning {
    pub r#type: String,
    pub message: String,
    pub file: String,
}

pub(crate) fn run(args: TargetsArgs) -> Result<ExitCode> {
    let report = generate_targets(&args)?;
    let format = if args.json {
        TargetsFormat::Json
    } else {
        args.format.unwrap_or(TargetsFormat::Json)
    };
    if matches!(format, TargetsFormat::Commands) && !report.warnings.is_empty() {
        let warnings = report
            .warnings
            .iter()
            .map(|warning| format!("warning: {}: {}", warning.file, warning.message))
            .collect::<Vec<_>>()
            .join("\n");
        bail!(
            "{warnings}\n`tests targets --format commands` requires all requested files to be owned by a configured test framework.\n"
        );
    }
    crate::invocation::check_timeout()?;
    print!("{}", render_targets(&report, format)?);
    Ok(ExitCode::SUCCESS)
}

pub(crate) fn generate_targets(args: &TargetsArgs) -> Result<TestsTargetsReport> {
    let cwd = std::env::current_dir().context("cwd must be accessible")?;
    let root = no_mistakes::cli::resolve_optional_root(Some(&args.root), &cwd);
    let root = no_mistakes::codebase::ts_resolver::normalize_path(&root);
    let root = root.canonicalize().unwrap_or(root);
    let config = load_v2_config(&root, args.config.as_deref())?;
    let discovered = discover_tests(&root, &config, runner_for(args.framework))?;

    let by_path: BTreeMap<PathBuf, Vec<TestExecutionTarget>> =
        discovered.targets_by_path.into_iter().collect();
    let mut tests = Vec::new();
    let mut warnings = Vec::new();
    for raw in &args.files {
        let abs = resolve_input_file(&root, raw);
        let rel = relative_slash(&root, &abs);
        if let Some(targets) = by_path.get(&abs) {
            tests.push(TestTargetRow {
                test_file: rel,
                targets: targets.clone(),
            });
        } else {
            warnings.push(TargetWarning {
                r#type: "unmatched-test".to_string(),
                message: format!(
                    "`{}` is not owned by a {} test project",
                    rel,
                    framework_name(args.framework)
                ),
                file: rel,
            });
        }
    }
    tests.sort_by(|a, b| a.test_file.cmp(&b.test_file));
    warnings.sort_by(|a, b| a.file.cmp(&b.file));
    Ok(TestsTargetsReport {
        framework: framework_name(args.framework).to_string(),
        tests,
        warnings,
    })
}

pub(crate) fn commands_for_plan(plan: &TestPlan) -> Vec<String> {
    let mut commands = Vec::new();
    for test in &plan.selected_tests {
        for target in &test.targets {
            commands.push(shell_join(&target_command(target)));
        }
    }
    commands.sort();
    commands.dedup();
    commands
}

pub(crate) fn ensure_plan_commands_available(plan: &TestPlan, command: &str) -> Result<()> {
    if plan
        .selected_tests
        .iter()
        .any(|test| test.targets.is_empty())
    {
        bail!(
            "`{command} --format commands` requires selected tests to include framework execution targets.\n"
        );
    }
    Ok(())
}

fn render_targets(report: &TestsTargetsReport, format: TargetsFormat) -> Result<String> {
    Ok(match format {
        TargetsFormat::Json => format!("{}\n", serde_json::to_string_pretty(report)?),
        TargetsFormat::Commands => {
            let mut commands = Vec::new();
            for row in &report.tests {
                for target in &row.targets {
                    commands.push(shell_join(&target_command(target)));
                }
            }
            commands.sort();
            commands.dedup();
            commands
                .into_iter()
                .map(|command| format!("{command}\n"))
                .collect()
        }
        TargetsFormat::Paths => report
            .tests
            .iter()
            .map(|row| format!("{}\n", row.test_file))
            .collect(),
        TargetsFormat::Human => {
            let mut out = String::new();
            for row in &report.tests {
                out.push_str(&format!("{}\n", row.test_file));
                for target in &row.targets {
                    out.push_str(&format!("  {}\n", shell_join(&target_command(target))));
                }
            }
            for warning in &report.warnings {
                out.push_str(&format!("warning: {}: {}\n", warning.file, warning.message));
            }
            out
        }
    })
}

fn target_command(target: &TestExecutionTarget) -> Vec<String> {
    let mut command = target.base_command.clone();
    command.extend(target.runner_args.iter().cloned());
    command
}

pub(crate) fn shell_join(command: &[String]) -> String {
    command
        .iter()
        .map(|token| shell_quote(token))
        .collect::<Vec<_>>()
        .join(" ")
}

fn shell_quote(token: &str) -> String {
    let safe = !token.is_empty()
        && token.chars().all(|c| {
            c.is_ascii_alphanumeric()
                || matches!(c, '_' | '-' | '.' | '/' | ':' | '=' | '@' | ',' | '+')
        });
    if safe {
        token.to_string()
    } else {
        format!("'{}'", token.replace('\'', "'\\''"))
    }
}

fn resolve_input_file(root: &Path, raw: &Path) -> PathBuf {
    let path = if raw.is_absolute() {
        raw.to_path_buf()
    } else {
        root.join(raw)
    };
    no_mistakes::codebase::ts_resolver::normalize_path(&path)
}

fn relative_slash(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn runner_for(framework: TestFramework) -> TestRunner {
    match framework {
        TestFramework::Dotnet => TestRunner::Dotnet,
        TestFramework::Vitest => TestRunner::Vitest,
        TestFramework::Playwright => TestRunner::Playwright,
        TestFramework::Swift => TestRunner::Swift,
    }
}

fn framework_name(framework: TestFramework) -> &'static str {
    match framework {
        TestFramework::Dotnet => "dotnet",
        TestFramework::Vitest => "vitest",
        TestFramework::Playwright => "playwright",
        TestFramework::Swift => "swift",
    }
}

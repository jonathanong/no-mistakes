//! Report generation for `impacted-checks`: reuse the test-plan engine per
//! framework and apply the configured generic checks.

use super::frameworks::framework_present;
use super::{CheckCommand, CheckKind, ImpactedChecksArgs, ImpactedChecksReport};
use crate::config::v2::load_v2_config;
use crate::config::v2::schema::{CheckFileArgs, NoMistakesConfig};
use crate::tests::{PlanArgs, TestFramework, Warning};
use anyhow::Result;
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// Compute the impacted-checks report (shared by the CLI and N-API).
pub fn generate_impacted_checks(args: &ImpactedChecksArgs) -> Result<ImpactedChecksReport> {
    let cwd = std::env::current_dir()?;
    let root = crate::cli::resolve_optional_root(Some(&args.root), &cwd);
    let root = crate::codebase::ts_resolver::normalize_path(&root);
    let root = root.canonicalize().unwrap_or(root);
    let config = load_v2_config(&root, args.config.as_deref())?;

    let plan_args = plan_args_for(args, None);
    let collected = crate::tests::changed_files::collect_changed_files(&plan_args, &root)?;
    let changed_files: Vec<String> = sorted_unique(
        collected
            .files
            .iter()
            .map(|file| relative_slash(&root, file)),
    );
    // Files that no longer exist on disk — deleted via `--diff`, removed on the
    // `--base` branch (whose `git diff --name-only` omits status), or a missing
    // `--changed-file`. Per-file (append) checks must skip these so they never
    // target a nonexistent path; whole-project checks still trigger.
    let missing: BTreeSet<String> = collected
        .files
        .iter()
        .filter(|file| !file.exists())
        .map(|file| relative_slash(&root, file))
        .collect();

    let mut checks: Vec<CheckCommand> = Vec::new();
    let mut warnings: Vec<Warning> = Vec::new();
    let mut fallback_triggered = false;

    // Only run frameworks the repo actually uses. The test-plan engine's
    // discovery fallback classifies conventional `*.test.*` files as Vitest even
    // when Vitest is absent, so without this gate a Jest/Mocha repo would emit
    // spurious `vitest` commands. A framework counts as present when it is
    // explicitly configured or its config file exists at the repo root.
    for framework in [
        TestFramework::Vitest,
        TestFramework::Playwright,
        TestFramework::Swift,
    ] {
        if !framework_present(&root, &config, framework) {
            continue;
        }
        let framework_args = plan_args_for(args, Some(framework));
        let plan = crate::tests::plan::generate_plan(&framework_args)?;
        fallback_triggered |= plan.fallback_triggered;
        warnings.extend(plan.warnings.iter().cloned());
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

    checks.extend(generic_checks(&config, &changed_files, &missing)?);

    Ok(ImpactedChecksReport {
        changed_files,
        checks: dedupe_checks(checks),
        warnings: dedupe_warnings(warnings),
        fallback_triggered,
    })
}

pub(super) fn generic_checks(
    config: &NoMistakesConfig,
    changed_files: &[String],
    missing: &BTreeSet<String>,
) -> Result<Vec<CheckCommand>> {
    let mut out = Vec::new();
    for def in &config.checks.commands {
        let include = build_globset(&def.include)?;
        let exclude = build_globset(&def.exclude)?;
        let matched: Vec<String> = changed_files
            .iter()
            .filter(|file| {
                include.as_ref().is_some_and(|set| set.is_match(file))
                    && exclude.as_ref().is_none_or(|set| !set.is_match(file))
            })
            .cloned()
            .collect();
        if matched.is_empty() {
            continue;
        }
        let mut command = def.command.clone();
        let files = if def.file_args == CheckFileArgs::Append {
            // Append only surviving files — a per-file command (e.g. eslint) on a
            // deleted path would fail. Whole-project checks still trigger below.
            let appended: Vec<String> = matched
                .into_iter()
                .filter(|f| !missing.contains(f))
                .collect();
            if appended.is_empty() {
                continue;
            }
            command.extend(appended.iter().cloned());
            appended
        } else {
            matched
        };
        out.push(CheckCommand {
            name: def.name.clone(),
            kind: CheckKind::Generic,
            command,
            files,
        });
    }
    Ok(out)
}

fn build_globset(patterns: &[String]) -> Result<Option<GlobSet>> {
    if patterns.is_empty() {
        return Ok(None);
    }
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        // Strip leading `./` (and resolve `.`/`..`) so a root-relative pattern
        // like `./src/**/*.ts` matches repo-relative paths such as `src/foo.ts`.
        let pattern = crate::codebase::glob_normalize::normalize(pattern);
        builder.add(Glob::new(&pattern)?);
    }
    Ok(Some(builder.build()?))
}

fn plan_args_for(args: &ImpactedChecksArgs, framework: Option<TestFramework>) -> PlanArgs {
    let mut changed_file = args.changed_file.clone();
    changed_file.extend(args.files.iter().cloned());
    PlanArgs {
        framework,
        root: args.root.clone(),
        config: args.config.clone(),
        tsconfig: args.tsconfig.clone(),
        base: args.base.clone(),
        head: args.head.clone(),
        changed_file,
        changed_files: args.changed_files.clone(),
        diff: args.diff.clone(),
        diff_stdin: false,
        diff_command: None,
        entrypoints: Vec::new(),
        entrypoint_symbols: Vec::new(),
        include_symbols: false,
        diff_content: args.diff_content.clone(),
        environment: "pre-push".to_string(),
        limit_percent: None,
        limit_files: None,
        global_config_fallback: None,
        format: None,
        json: false,
    }
}

pub(super) fn dedupe_checks(checks: Vec<CheckCommand>) -> Vec<CheckCommand> {
    // Collapse checks sharing the same command, merging their triggering files
    // so the report lists every file that caused the command (not just the first).
    let mut by_command: BTreeMap<Vec<String>, CheckCommand> = BTreeMap::new();
    for mut check in checks {
        match by_command.get_mut(&check.command) {
            Some(existing) => existing.files.append(&mut check.files),
            None => {
                by_command.insert(check.command.clone(), check);
            }
        }
    }
    let mut unique: Vec<CheckCommand> = by_command.into_values().collect();
    for check in &mut unique {
        check.files.sort();
        check.files.dedup();
    }
    unique.sort_by(|a, b| a.command.cmp(&b.command));
    unique
}

pub(super) fn dedupe_warnings(warnings: Vec<Warning>) -> Vec<Warning> {
    let mut seen = BTreeSet::new();
    let mut unique: Vec<Warning> = warnings
        .into_iter()
        .filter(|warning| seen.insert((warning.r#type.clone(), warning.file.clone())))
        .collect();
    unique.sort_by(|a, b| (&a.file, &a.message).cmp(&(&b.file, &b.message)));
    unique
}

fn sorted_unique(values: impl Iterator<Item = String>) -> Vec<String> {
    let set: BTreeSet<String> = values.collect();
    set.into_iter().collect()
}

fn relative_slash(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

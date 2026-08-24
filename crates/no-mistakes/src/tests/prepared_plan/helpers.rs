use super::{LockfileAnalysis, Path, PlanArgs, Result, TestFramework, TestRunner};
use no_mistakes::codebase::dependencies::graph::GraphBuildPlan;

pub(super) fn resolve_args(args: &PlanArgs) -> Result<PlanArgs> {
    if args.direct_test_owner && args.framework.is_none() {
        anyhow::bail!("--direct-test-owner requires a framework (for example, `tests plan vitest --direct-test-owner`) because direct ownership requires framework-specific test ownership");
    }
    if args.direct_test_owner && !args.entrypoints.is_empty() {
        anyhow::bail!("--direct-test-owner conflicts with --entrypoint: direct-owner selection only follows changed files and one reverse canonical graph edge; use `tests impact` for explicit entrypoint traversal");
    }
    if args.direct_test_owner
        && (args.limit_percent.is_some()
            || args.limit_files.is_some()
            || args.global_config_fallback.is_some())
    {
        anyhow::bail!("--direct-test-owner conflicts with --limit-percent, --limit-files, and --global-config-fallback; remove those policy overrides because direct ownership bypasses configured plan policy");
    }
    if args.from_git_diff.is_some() && (args.base.is_some() || args.head.is_some()) {
        anyhow::bail!("--from-git-diff conflicts with --base/--head; provide only one");
    }
    let mut args = args.clone();
    if let Some(spec) = args.from_git_diff.take() {
        let (base, head) = super::super::changed_files::parse_git_diff_refspec(&spec)?;
        args.base = Some(base);
        args.head = Some(head.unwrap_or_else(|| "HEAD".to_string()));
    }
    Ok(args)
}

pub(super) fn lockfile_packages(
    root: &Path,
    analysis: &LockfileAnalysis,
) -> Vec<(String, String, Vec<std::path::PathBuf>)> {
    analysis
        .diff_by_lockfile
        .iter()
        .flat_map(|(lockfile_path, diff)| {
            let relative = super::super::plan::relative_path(root, lockfile_path);
            diff.all_changed_names()
                .map(|name| {
                    let scopes = analysis
                        .pnpm_importer_paths
                        .get(lockfile_path)
                        .and_then(|paths| paths.get(name))
                        .into_iter()
                        .flatten()
                        .map(|path| {
                            lockfile_path
                                .parent()
                                .unwrap_or(root)
                                .join(path)
                                .join("package.json")
                        })
                        .collect();
                    (name.to_string(), relative.clone(), scopes)
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

pub(super) fn test_runner(framework: TestFramework) -> TestRunner {
    match framework {
        TestFramework::Dotnet => TestRunner::Dotnet,
        TestFramework::Playwright => TestRunner::Playwright,
        TestFramework::Vitest => TestRunner::Vitest,
        TestFramework::Swift => TestRunner::Swift,
        TestFramework::Python => TestRunner::Python,
        TestFramework::Go => TestRunner::Go,
        TestFramework::Cargo => TestRunner::Cargo,
        TestFramework::Rails => TestRunner::Rails,
        TestFramework::Php => TestRunner::Php,
        TestFramework::Java => TestRunner::Java,
        TestFramework::Kotlin => TestRunner::Kotlin,
        TestFramework::Elixir => TestRunner::Elixir,
        TestFramework::Dart => TestRunner::Dart,
        TestFramework::Jest => TestRunner::Jest,
    }
}

pub(super) fn framework_graph_plan(framework: TestFramework) -> GraphBuildPlan {
    let mut plan = GraphBuildPlan::test_impact();
    plan.dotnet = framework == TestFramework::Dotnet;
    plan.swift = framework == TestFramework::Swift;
    let playwright = framework == TestFramework::Playwright;
    plan.playwright_routes = playwright;
    plan.playwright_selectors = playwright;
    plan
}

pub(super) fn test_framework(runner: TestRunner) -> TestFramework {
    match runner {
        TestRunner::Dotnet => TestFramework::Dotnet,
        TestRunner::Playwright => TestFramework::Playwright,
        TestRunner::Vitest => TestFramework::Vitest,
        TestRunner::Swift => TestFramework::Swift,
        TestRunner::Python => TestFramework::Python,
        TestRunner::Go => TestFramework::Go,
        TestRunner::Cargo => TestFramework::Cargo,
        TestRunner::Rails => TestFramework::Rails,
        TestRunner::Php => TestFramework::Php,
        TestRunner::Java => TestFramework::Java,
        TestRunner::Kotlin => TestFramework::Kotlin,
        TestRunner::Elixir => TestFramework::Elixir,
        TestRunner::Dart => TestFramework::Dart,
        TestRunner::Jest => TestFramework::Jest,
    }
}

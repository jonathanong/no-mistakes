use super::super::ImpactedChecksArgs;
use crate::tests::{PlanArgs, TestFramework};

pub(super) fn discover_phase(framework: TestFramework) -> &'static str {
    match framework {
        TestFramework::Dotnet => "discover.dotnet",
        TestFramework::Vitest => "discover.vitest",
        TestFramework::Playwright => "discover.playwright",
        TestFramework::Swift => "discover.swift",
    }
}

pub(super) fn select_phase(framework: TestFramework) -> &'static str {
    match framework {
        TestFramework::Dotnet => "select.dotnet",
        TestFramework::Vitest => "select.vitest",
        TestFramework::Playwright => "select.playwright",
        TestFramework::Swift => "select.swift",
    }
}

pub(crate) fn plan_args_for(
    args: &ImpactedChecksArgs,
    framework: Option<TestFramework>,
) -> PlanArgs {
    let mut changed_file = args.changed_file.clone();
    changed_file.extend(args.files.iter().cloned());
    PlanArgs {
        framework,
        root: args.root.clone(),
        config: args.config.clone(),
        tsconfig: args.tsconfig.clone(),
        base: args.base.clone(),
        head: args.head.clone(),
        from_git_diff: None,
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

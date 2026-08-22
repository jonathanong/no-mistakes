use super::super::ImpactedChecksArgs;
use crate::tests::{PlanArgs, TestFramework};

pub(super) fn discover_phase(framework: TestFramework) -> &'static str {
    match framework {
        TestFramework::Dotnet => "discover.dotnet",
        TestFramework::Vitest => "discover.vitest",
        TestFramework::Playwright => "discover.playwright",
        TestFramework::Swift => "discover.swift",
        TestFramework::Python => "discover.python",
        TestFramework::Go => "discover.go",
        TestFramework::Cargo => "discover.cargo",
        TestFramework::Rails => "discover.rails",
        TestFramework::Php => "discover.php",
        TestFramework::Java => "discover.java",
        TestFramework::Kotlin => "discover.kotlin",
        TestFramework::Elixir => "discover.elixir",
        TestFramework::Dart => "discover.dart",
        TestFramework::Jest => "discover.jest",
    }
}

pub(super) fn select_phase(framework: TestFramework) -> &'static str {
    match framework {
        TestFramework::Dotnet => "select.dotnet",
        TestFramework::Vitest => "select.vitest",
        TestFramework::Playwright => "select.playwright",
        TestFramework::Swift => "select.swift",
        TestFramework::Python => "select.python",
        TestFramework::Go => "select.go",
        TestFramework::Cargo => "select.cargo",
        TestFramework::Rails => "select.rails",
        TestFramework::Php => "select.php",
        TestFramework::Java => "select.java",
        TestFramework::Kotlin => "select.kotlin",
        TestFramework::Elixir => "select.elixir",
        TestFramework::Dart => "select.dart",
        TestFramework::Jest => "select.jest",
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
        diff_stdin: args.diff_stdin,
        diff_command: args.diff_command.clone(),
        entrypoints: Vec::new(),
        entrypoint_symbols: Vec::new(),
        include_symbols: false,
        diff_content: args.diff_content.clone(),
        environment: "pre-push".to_string(),
        limit_percent: None,
        limit_files: None,
        global_config_fallback: None,
        direct_test_owner: false,
        format: None,
        json: false,
        include_comment: false,
    }
}

#[cfg(test)]
#[path = "args/tests.rs"]
mod tests;

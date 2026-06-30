use serde::{Deserialize, Serialize};

use super::types::TestRunner;

#[cfg(test)]
mod tests;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TestExecutionTarget {
    pub runner: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    pub base_command: Vec<String>,
    pub runner_args: Vec<String>,
}

pub(super) fn target_for(
    runner: TestRunner,
    config: Option<&str>,
    project: Option<&str>,
    test_file: &str,
) -> TestExecutionTarget {
    if runner == TestRunner::Dotnet {
        return dotnet_target_for(config, project, test_file);
    }
    if runner == TestRunner::Swift {
        return swift_target_for(config, project, test_file);
    }

    let mut runner_args = Vec::new();
    if let Some(config) = config {
        runner_args.push("--config".to_string());
        runner_args.push(config.to_string());
    }
    if let Some(project) = project {
        runner_args.push("--project".to_string());
        runner_args.push(project.to_string());
    }
    runner_args.push(test_file_arg(runner, test_file));

    let base_command = if runner == TestRunner::Playwright {
        vec!["playwright".to_string(), "test".to_string()]
    } else {
        vec!["vitest".to_string()]
    };

    TestExecutionTarget {
        runner: runner.as_str().to_string(),
        config: config.map(str::to_string),
        project: project.map(str::to_string),
        base_command,
        runner_args,
    }
}

fn dotnet_target_for(
    project_path: Option<&str>,
    project: Option<&str>,
    _test_file: &str,
) -> TestExecutionTarget {
    let mut runner_args = Vec::new();
    if let Some(project_path) = project_path {
        runner_args.push(project_path.to_string());
    }
    runner_args.push("--no-restore".to_string());

    TestExecutionTarget {
        runner: TestRunner::Dotnet.as_str().to_string(),
        config: project_path.map(str::to_string),
        project: project.map(str::to_string),
        base_command: vec!["dotnet".to_string(), "test".to_string()],
        runner_args,
    }
}

fn swift_target_for(
    package: Option<&str>,
    project: Option<&str>,
    test_file: &str,
) -> TestExecutionTarget {
    let mut runner_args = Vec::new();
    if let Some(package) = package {
        runner_args.push("--package-path".to_string());
        runner_args.push(package.to_string());
    }
    runner_args.push("--filter".to_string());
    runner_args.push(
        project
            .unwrap_or_else(|| swift_filter_from_path(test_file))
            .to_string(),
    );

    TestExecutionTarget {
        runner: TestRunner::Swift.as_str().to_string(),
        config: package.map(str::to_string),
        project: project.map(str::to_string),
        base_command: vec!["swift".to_string(), "test".to_string()],
        runner_args,
    }
}

fn swift_filter_from_path(test_file: &str) -> &str {
    test_file.split('/').rev().nth(1).unwrap_or(test_file)
}

fn test_file_arg(runner: TestRunner, test_file: &str) -> String {
    match runner {
        TestRunner::Dotnet => test_file.to_string(),
        TestRunner::Playwright => regex_escape(test_file),
        TestRunner::Vitest => test_file.to_string(),
        TestRunner::Swift => test_file.to_string(),
    }
}

fn regex_escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        if matches!(
            ch,
            '\\' | '.' | '+' | '*' | '?' | '(' | ')' | '|' | '[' | ']' | '{' | '}' | '^' | '$'
        ) {
            escaped.push('\\');
        }
        escaped.push(ch);
    }
    escaped
}

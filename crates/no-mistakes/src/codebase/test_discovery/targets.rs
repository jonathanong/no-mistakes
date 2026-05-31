use serde::{Deserialize, Serialize};

use super::types::TestRunner;

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
    let mut runner_args = Vec::new();
    if let Some(config) = config {
        runner_args.push("--config".to_string());
        runner_args.push(config.to_string());
    }
    if let Some(project) = project {
        runner_args.push("--project".to_string());
        runner_args.push(project.to_string());
    }
    runner_args.push(test_file.to_string());

    TestExecutionTarget {
        runner: runner.as_str().to_string(),
        config: config.map(str::to_string),
        project: project.map(str::to_string),
        base_command: match runner {
            TestRunner::Playwright => vec!["playwright".to_string(), "test".to_string()],
            TestRunner::Vitest => vec!["vitest".to_string()],
        },
        runner_args,
    }
}

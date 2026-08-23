use super::{framework_plan, test_runner, TestFramework};
use anyhow::Result;
use no_mistakes::codebase::test_discovery::TestRunner;
use no_mistakes::config::v2::schema::{NoMistakesConfig, TestPlanProjectDependency};

pub(super) fn validate_targeted_targets(
    config: &NoMistakesConfig,
    framework: TestFramework,
    prepared: &crate::tests::prepared_plan::PreparedTestPlanRequest,
) -> Result<()> {
    let runner = test_runner(framework);
    let projects = prepared.requested_runner_projects(runner)?;
    for (resource_project, dependency) in &framework_plan(config, framework)
        .full_suite_triggers
        .projects
    {
        let TestPlanProjectDependency::Targeted(targeted) = dependency else {
            continue;
        };
        validate_runner_targets(
            &targeted.targets,
            &projects,
            runner,
            prepared,
            &format!("projects.{resource_project}"),
        )?;
    }
    for (index, trigger) in framework_plan(config, framework)
        .full_suite_triggers
        .triggers
        .iter()
        .enumerate()
    {
        if trigger.targets.is_empty() {
            continue;
        }
        validate_runner_targets(
            &trigger.targets,
            &projects,
            runner,
            prepared,
            &format!("triggers[{index}]"),
        )?;
    }
    Ok(())
}

fn validate_runner_targets(
    targets: &[String],
    projects: &[no_mistakes::codebase::test_discovery::PreparedRunnerProject],
    runner: TestRunner,
    prepared: &crate::tests::prepared_plan::PreparedTestPlanRequest,
    field_suffix: &str,
) -> Result<()> {
    for (index, target) in targets.iter().enumerate() {
        let matching = projects
            .iter()
            .filter(|project| project.runner_project_arg.as_deref() == Some(target.as_str()))
            .collect::<Vec<_>>();
        let field = format!(
            "{}.testPlan.{}.fullSuiteTriggers.{field_suffix}.targets[{index}]",
            prepared.config_path().map_or_else(
                || "<in-memory config>".to_string(),
                |path| path.display().to_string()
            ),
            runner.as_str()
        );
        match matching.len() {
            1 => {}
            0 => anyhow::bail!(
                "{field} references unknown {} runner project `{target}`",
                runner.as_str()
            ),
            _ => anyhow::bail!(
                "{field} references ambiguous {} runner project `{target}` in configs: {}",
                runner.as_str(),
                matching
                    .iter()
                    .map(|project| project.config.as_deref().unwrap_or("<default>"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }
    Ok(())
}

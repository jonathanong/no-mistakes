use super::{project_dependency_patterns, project_relative_pattern, CoverageSource, CoverageUnit};
use crate::config::v2::schema::{NoMistakesConfig, TestPlanProjectDependency};

pub(super) fn push_full_suite_trigger_units(
    config: &NoMistakesConfig,
    units: &mut Vec<CoverageUnit>,
) {
    for (project_name, trigger) in &config.test_plan.vitest.full_suite_triggers.projects {
        let Some(project) = config.projects.get(project_name) else {
            continue;
        };
        let patterns = project_dependency_patterns(project_name, project, trigger);
        match trigger {
            TestPlanProjectDependency::Targeted(targeted) => {
                for target in &targeted.targets {
                    units.push(CoverageUnit {
                        project: target.clone(),
                        source: CoverageSource::FullSuiteTrigger,
                        patterns: patterns.clone(),
                    });
                }
            }
            TestPlanProjectDependency::All(_) | TestPlanProjectDependency::Patterns(_) => {
                units.push(CoverageUnit {
                    project: project_name.clone(),
                    source: CoverageSource::FullSuiteTrigger,
                    patterns,
                });
            }
        }
    }
    for trigger in &config.test_plan.vitest.full_suite_triggers.triggers {
        let patterns = trigger
            .paths
            .iter()
            .map(|pattern| project_relative_pattern(".", pattern))
            .collect::<Vec<_>>();
        // Targeted names are aliases for the runner projects they select.
        // Framework-wide triggers (empty targets) keep the trigger name as
        // the coverage unit so `projectFilters` can key off that name.
        let projects = if trigger.targets.is_empty() {
            vec![trigger.name.clone()]
        } else {
            trigger.targets.clone()
        };
        for project in projects {
            units.push(CoverageUnit {
                project,
                source: CoverageSource::FullSuiteTrigger,
                patterns: patterns.clone(),
            });
        }
    }
}

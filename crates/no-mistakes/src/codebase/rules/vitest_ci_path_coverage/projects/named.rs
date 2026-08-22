use super::{
    normalize_project_glob_part, project_dependency_patterns, CoverageSource, CoverageUnit,
};
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
            .map(|pattern| normalize_project_glob_part(pattern))
            .collect::<Vec<_>>();
        units.push(CoverageUnit {
            project: trigger.name.clone(),
            source: CoverageSource::FullSuiteTrigger,
            patterns: patterns.clone(),
        });
        for target in &trigger.targets {
            if target == &trigger.name {
                continue;
            }
            units.push(CoverageUnit {
                project: target.clone(),
                source: CoverageSource::FullSuiteTrigger,
                patterns: patterns.clone(),
            });
        }
    }
}

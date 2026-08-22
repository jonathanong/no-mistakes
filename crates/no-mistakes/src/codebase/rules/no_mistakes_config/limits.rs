use super::finding;
use super::paths::frameworks;
use crate::codebase::rules::RuleFinding;
use crate::config::v2::schema::{TestPlanEnvironment, TestPlanGroupType, TestPlanLimit};
use crate::config::v2::NoMistakesConfig;

pub(super) fn lint(config: &NoMistakesConfig, config_file: &str) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    for (framework, plan) in frameworks(config) {
        for (env_name, env) in &plan.environments {
            if has_effective_limit(&env.limit) && has_direct_group(env) {
                findings.push(finding(
                    config_file,
                    format!(
                        "testPlan.{framework}.environments.{env_name} has a limit while a direct group exists; \
scope the budget onto non-direct groups so changed tests are not dropped (regression #9440)"
                    ),
                ));
            }
        }
    }
    findings
}

fn has_direct_group(env: &TestPlanEnvironment) -> bool {
    env.groups.is_empty()
        || env
            .groups
            .iter()
            .any(|group| group.type_ == TestPlanGroupType::Direct)
}

fn has_effective_limit(limit: &Option<TestPlanLimit>) -> bool {
    let Some(limit) = limit else {
        return false;
    };
    limit.files.is_some()
        || limit
            .percent
            .as_ref()
            .and_then(|percent| percent.value())
            .is_some()
}

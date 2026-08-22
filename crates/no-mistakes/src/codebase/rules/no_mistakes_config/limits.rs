use super::finding;
use super::paths::frameworks;
use crate::codebase::rules::RuleFinding;
use crate::config::v2::schema::TestPlanGroupType;
use crate::config::v2::NoMistakesConfig;

pub(super) fn lint(config: &NoMistakesConfig, config_file: &str) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    for (framework, plan) in frameworks(config) {
        for (env_name, env) in &plan.environments {
            let has_direct = env
                .groups
                .iter()
                .any(|group| group.type_ == TestPlanGroupType::Direct);
            if env.limit.is_some() && has_direct {
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

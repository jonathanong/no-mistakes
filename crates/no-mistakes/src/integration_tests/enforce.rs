use super::types::{EffectiveIntegrationPolicy, IntegrationFinding, Suite, TestCase};
use crate::codebase::ts_source::relative_slash_path;
use std::path::Path;

pub(super) fn enforce_policy(
    root: &Path,
    suite: &Suite,
    test: &TestCase,
    integrations: &[String],
) -> Vec<IntegrationFinding> {
    match &suite.policy {
        EffectiveIntegrationPolicy::AllowedIntegrations {
            integrations: allowed_integrations,
        } => enforce_suite_policy(root, suite, test, integrations, allowed_integrations),
    }
}

fn enforce_suite_policy(
    root: &Path,
    suite: &Suite,
    test: &TestCase,
    integrations: &[String],
    allowed_integrations: &[String],
) -> Vec<IntegrationFinding> {
    let mut findings = Vec::new();
    for name in integrations {
        if allowed_integrations.contains(name) {
            continue;
        }
        findings.push(finding(
            root,
            suite,
            test,
            Some(name.clone()),
            format!(
                "{} suite {} allows only integration={}; found integration={name}",
                suite.framework.as_str(),
                suite.name,
                allowed_integrations.join(",")
            ),
        ));
    }
    findings
}

fn finding(
    root: &Path,
    suite: &Suite,
    test: &TestCase,
    integration: Option<String>,
    message: String,
) -> IntegrationFinding {
    IntegrationFinding {
        framework: suite.framework.as_str().to_string(),
        suite: suite.name.clone(),
        file: relative_slash_path(root, &test.function_key.file),
        line: test.line,
        test_name: test.name.clone(),
        describe_path: test.describe_path.clone(),
        integration,
        message,
    }
}

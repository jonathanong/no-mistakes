use super::{types::Component, RuleFinding, RULE_ID};

pub(super) fn unanalyzable(component: Component) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: component.repo_file,
        line: component.line,
        message: format!(
            "Storybook coverage of `{}` is unanalyzable at effort low: transitive rendering and dynamic/mock boundaries are not followed. Import and use the component directly in a configured story, enable allowColocatedTests and add a matching colocated test, or use effort max to infer coverage.",
            component.export_name
        ),
        import: None,
        target: Some("unanalyzable".to_string()),
    }
}

pub(super) fn missing(component: Component) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: component.repo_file,
        line: component.line,
        message: format!(
            "React component `{}` is selected for Storybook coverage but no reachable story imports it or a parent component that renders it. Add a Storybook story, add an accepted colocated test when `allow_colocated_tests` is enabled, render it through a covered parent component, exclude it from `{RULE_ID}`, or add a documented no-mistakes disable comment.",
            component.export_name
        ),
        import: None,
        target: Some(component.key),
    }
}

/// Add Storybook demand without clearing another consumer's explicit projection.
#[doc(hidden)]
pub fn configure_effort_plan(
    config: &crate::config::v2::NoMistakesConfig,
    plan: &mut crate::codebase::check_facts::CheckFactPlan,
) -> anyhow::Result<()> {
    for rule in config.rule_applications(RULE_ID) {
        plan.dynamic_imports |=
            config.effective_rule_analysis(rule)?.effort == crate::config::v2::Effort::Max;
    }
    Ok(())
}

pub(crate) fn prepare_low_config(
    root: &std::path::Path,
    config: &crate::config::v2::NoMistakesConfig,
    files: &[std::path::PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> anyhow::Result<Option<crate::config::v2::NoMistakesConfig>> {
    let low_requested = config.rule_applications(RULE_ID).iter().any(|rule| {
        config
            .requested_rule_analysis(rule)
            .is_ok_and(|policy| policy.effort == crate::config::v2::Effort::Low)
    });
    if !low_requested {
        return Ok(None);
    }
    let mut config = config.clone();
    // Standalone preparation must not validate or change unrelated applications.
    let mut scoped = config.clone();
    scoped.rules.retain(|rule| rule.rule == RULE_ID);
    super::super::analysis_policy::prepare_rule_analysis(root, &mut scoped, files, sources)?;
    for (rule, prepared) in config
        .rules
        .iter_mut()
        .filter(|rule| rule.rule == RULE_ID)
        .zip(scoped.rules)
    {
        rule.resolved_analysis = prepared.resolved_analysis;
    }
    Ok(Some(config))
}

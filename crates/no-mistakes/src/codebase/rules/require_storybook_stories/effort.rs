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

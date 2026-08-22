use super::{rule_targets, NoMistakesConfig, RuleDef, RuleScope};

impl RuleDef {
    pub fn rule_options<T: for<'de> serde::Deserialize<'de> + Default>(&self) -> T {
        serde_yaml::from_value(self.options.clone()).unwrap_or_default()
    }

    pub fn applies_to_project(&self, project: &str) -> bool {
        self.enabled && self.projects.iter().any(|name| name == project)
    }

    pub fn applies_to_repository(&self) -> bool {
        self.enabled && self.scope == Some(RuleScope::Repository)
    }
}

impl NoMistakesConfig {
    pub fn rule_applications<'a>(&'a self, rule_id: &str) -> Vec<&'a RuleDef> {
        self.rules
            .iter()
            .filter(move |rule| {
                rule.enabled && rule.rule == rule_id && self.rule_has_effective_target(rule)
            })
            .collect()
    }

    pub fn rule_configured(&self, rule_id: &str) -> bool {
        !self.rule_applications(rule_id).is_empty()
    }

    pub fn rule_application_options<T: for<'de> serde::Deserialize<'de> + Default>(
        &self,
        rule_id: &str,
    ) -> Vec<T> {
        self.rule_applications(rule_id)
            .into_iter()
            .map(RuleDef::rule_options)
            .collect()
    }

    fn rule_has_effective_target(&self, rule: &RuleDef) -> bool {
        rule.scope == Some(RuleScope::Repository)
            || rule
                .projects
                .iter()
                .any(|project| self.projects.contains_key(project))
            || rule_targets::rule_has_effective_test_target(rule)
            || rule_targets::rule_has_playwright_apps_target(rule, self)
    }
}

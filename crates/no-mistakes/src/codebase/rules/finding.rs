pub(crate) fn rule_enabled(config: &crate::config::v2::NoMistakesConfig, rule_id: &str) -> bool {
    config.rule_configured(rule_id)
}

pub(crate) fn prepare_requested_rule_config(
    root: &std::path::Path,
    rule_id: &str,
    config: &crate::config::v2::NoMistakesConfig,
    files: &[std::path::PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> anyhow::Result<Option<crate::config::v2::NoMistakesConfig>> {
    let low_requested = config.rule_applications(rule_id).iter().any(|rule| {
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
    scoped.rules.retain(|rule| rule.rule == rule_id);
    super::prepare_rule_analysis(root, &mut scoped, files, sources)?;
    for (rule, prepared) in config
        .rules
        .iter_mut()
        .filter(|rule| rule.rule == rule_id)
        .zip(scoped.rules)
    {
        rule.resolved_analysis = prepared.resolved_analysis;
        rule.options = prepared.options;
    }
    Ok(Some(config))
}

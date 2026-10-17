pub(super) fn validate(options: &Options, config: &NoMistakesConfig) -> Result<()> {
    if options.effects.is_empty() {
        bail!("{RULE_ID}: options.effects must select at least one configured effect kind");
    }
    if options
        .allow
        .iter()
        .any(|allow| allow.file.is_empty() || allow.line == Some(0))
    {
        bail!("{RULE_ID}: allow entries require a non-empty file and positive line when supplied");
    }
    for kind in &options.effects {
        let Some(effect) = config.effects.get(kind) else {
            bail!("{RULE_ID}: unknown effects kind `{kind}`");
        };
        crate::effect_provenance::validate(effect, kind)?;
        anyhow::ensure!(
            effect
                .targets
                .iter()
                .chain(&effect.returned_executors)
                .all(|target| {
                    !target.module.trim().is_empty() && !target.export.trim().is_empty()
                }),
            "effects kind `{kind}` targets require non-empty module and export"
        );
    }
    Ok(())
}

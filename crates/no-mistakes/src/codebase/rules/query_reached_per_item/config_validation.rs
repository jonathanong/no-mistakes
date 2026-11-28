pub(in crate::codebase::rules) fn validate(options: &Options, config: &NoMistakesConfig) -> Result<()> {
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
    if options.allow.iter().any(|allow| {
        allow
            .function
            .as_ref()
            .is_some_and(|name| name.trim().is_empty())
    }) {
        bail!("{RULE_ID}: allow entries require a non-empty function when supplied");
    }
    for language in &options.languages {
        if !super::languages::known(language) {
            bail!("{RULE_ID}: unknown options.languages value `{language}`");
        }
        if super::languages::root_entries(config, language).is_empty() {
            bail!(
                "{RULE_ID}: options.languages `{language}` requires a configured tests root"
            );
        }
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

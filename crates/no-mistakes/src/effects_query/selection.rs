pub(crate) fn selection_from_config(
    config: &crate::config::v2::NoMistakesConfig,
    kind: &str,
    categories: &[String],
) -> Result<EffectsSelection> {
    let Some(kind_config) = config.effects.get(kind) else {
        let available: Vec<&str> = config.effects.keys().map(String::as_str).collect();
        bail!(
            "unknown effects kind: {kind} (configured kinds: {})",
            if available.is_empty() {
                "<none>".to_string()
            } else {
                available.join(", ")
            }
        );
    };
    crate::effect_provenance::validate(kind_config, kind)?;
    let provenance = crate::effect_provenance::EffectProvenance::new(kind_config, categories);
    let mut names = HashMap::new();
    for (category, functions) in &kind_config.categories {
        if !categories.is_empty() && !categories.iter().any(|value| value == category) {
            continue;
        }
        for function in functions {
            names.insert(function.clone(), Some(category.clone()));
        }
    }
    if categories.is_empty() {
        for function in &kind_config.functions {
            names.entry(function.clone()).or_insert(None);
        }
    }
    for target in kind_config
        .targets
        .iter()
        .chain(&kind_config.returned_executors)
    {
        anyhow::ensure!(
            !target.module.trim().is_empty() && !target.export.trim().is_empty(),
            "effects kind `{kind}` targets require non-empty module and export"
        );
    }
    let mut targets: crate::fx::FxHashMap<
        String,
        crate::fx::FxHashMap<String, crate::config::v2::schema::EffectTargetConfig>,
    > = crate::fx::FxHashMap::default();
    for target in &kind_config.targets {
        if categories.is_empty()
            || target
                .category
                .as_ref()
                .is_some_and(|category| categories.contains(category))
        {
            targets
                .entry(target.module.clone())
                .or_default()
                .insert(target.export.clone(), target.clone());
        }
    }
    let mut returned: crate::fx::FxHashMap<
        String,
        crate::fx::FxHashMap<String, crate::config::v2::schema::EffectTargetConfig>,
    > = crate::fx::FxHashMap::default();
    for target in &kind_config.returned_executors {
        if categories.is_empty()
            || target
                .category
                .as_ref()
                .is_some_and(|category| categories.contains(category))
        {
            returned
                .entry(target.module.clone())
                .or_default()
                .insert(target.export.clone(), target.clone());
        }
    }
    if names.is_empty() && targets.is_empty() && returned.is_empty() && provenance.is_empty() {
        bail!("effects kind `{kind}` has no functions for the requested categories");
    }
    Ok(EffectsSelection {
        kind: kind.to_string(),
        names,
        provenance,
        targets,
        returned,
    })
}

pub(crate) fn selection_fact_functions(
    selection: &EffectsSelection,
) -> impl Iterator<Item = String> + '_ {
    selection.names.keys().cloned()
}

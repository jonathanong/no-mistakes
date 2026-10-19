use crate::config::v2::schema::{EffectKindConfig, EffectTargetConfig, NoMistakesConfig};
use anyhow::{bail, Result};

pub(super) fn apply(config: &mut NoMistakesConfig) -> Result<bool> {
    for (kind, effect) in &config.effects {
        if effect.project.is_some() && kind != "postgres" {
            bail!(
                "effects.{kind}.project is unsupported; only effects.postgres.project selects a PostgreSQL project scope"
            );
        }
    }
    let Some(project_name) = config
        .effects
        .get("postgres")
        .and_then(|effect| effect.project.clone())
    else {
        return Ok(false);
    };
    if config
        .effects
        .get("postgres")
        .is_some_and(|effect| effect.project_scope_expanded)
    {
        return Ok(false);
    }
    let project = config.projects.get(&project_name).ok_or_else(|| {
        anyhow::anyhow!("effects.postgres.project `{project_name}` is not a known project")
    })?;
    let postgres = project.postgres.as_ref().ok_or_else(|| {
        anyhow::anyhow!("effects.postgres.project `{project_name}` has no postgres block")
    })?;
    let executors = postgres.executors.clone().unwrap_or_default();
    if (!executors.factory_names.is_empty() || !executors.type_names.is_empty())
        && executors.import_specifier.trim().is_empty()
    {
        bail!(
            "effects.postgres.project `{project_name}` requires a non-empty postgres.executors.importSpecifier"
        );
    }
    let module = executors.import_specifier;
    let factories = executors.factory_names;
    let types = executors.type_names;
    let effect = config
        .effects
        .get_mut("postgres")
        .expect("postgres effect exists");
    for name in &factories {
        push_selector(effect, &project_name, &module, name, SelectorKind::Returned)?;
    }
    for name in &types {
        push_selector(effect, &project_name, &module, name, SelectorKind::Typed)?;
    }
    effect.project_scope_expanded = true;
    Ok(true)
}

enum SelectorKind {
    Returned,
    Typed,
}

fn push_selector(
    effect: &mut EffectKindConfig,
    project: &str,
    module: &str,
    export: &str,
    kind: SelectorKind,
) -> Result<()> {
    if lists_selector(effect, module, export) {
        bail!(
            "effects.postgres.project `{project}` repeats `{module}` export `{export}`; remove it from returnedExecutors and typedExecutors"
        );
    }
    let selector = EffectTargetConfig {
        module: module.to_string(),
        export: export.to_string(),
        category: None,
    };
    match kind {
        SelectorKind::Returned => effect.returned_executors.push(selector),
        SelectorKind::Typed => effect.typed_executors.push(selector),
    }
    Ok(())
}

fn lists_selector(effect: &EffectKindConfig, module: &str, export: &str) -> bool {
    effect
        .returned_executors
        .iter()
        .chain(&effect.typed_executors)
        .any(|selector| selector.module == module && selector.export == export)
}

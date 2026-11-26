use anyhow::Result;
use globset::GlobBuilder;
use std::collections::BTreeMap;

use super::super::schema::PlaywrightSelectorWrapper;

use crate::config::source_locations::ConfigPath;

#[derive(Debug)]
struct InvalidConfigField {
    origin: ConfigPath,
    message: String,
}

impl std::fmt::Display for InvalidConfigField {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for InvalidConfigField {}

pub(super) fn field_error(origin: ConfigPath, message: String) -> anyhow::Error {
    InvalidConfigField { origin, message }.into()
}

pub(super) fn validate_globs(
    patterns: &[String],
    label: &str,
    origin: impl Fn(usize) -> ConfigPath,
) -> Result<()> {
    for (index, pattern) in patterns.iter().enumerate() {
        GlobBuilder::new(pattern.trim_start_matches("./"))
            .literal_separator(false)
            .build()
            .map_err(|error| {
                field_error(
                    origin(index),
                    format!("{label}[{index}]: contains invalid glob `{pattern}`: {error}"),
                )
            })?;
    }
    Ok(())
}

pub(super) fn annotate_with_origin(
    error: anyhow::Error,
    origin: impl FnOnce(
        &crate::config::source_locations::ConfigPath,
    ) -> Option<(std::path::PathBuf, usize)>,
) -> anyhow::Error {
    let Some(invalid) = error.downcast_ref::<InvalidConfigField>() else {
        return error;
    };
    let Some((path, line)) = origin(&invalid.origin) else {
        return error;
    };
    anyhow::anyhow!("{}:{line}: {error}", path.display())
}

pub(super) fn validate_playwright_selector_wrappers(
    wrappers: &[PlaywrightSelectorWrapper],
) -> Result<()> {
    let mut arguments_by_export = BTreeMap::new();
    for (index, wrapper) in wrappers.iter().enumerate() {
        if wrapper.module.trim().is_empty() {
            anyhow::bail!("tests.playwright.selectors.wrappers[{index}].module must not be blank");
        }
        if wrapper.export.trim().is_empty() {
            anyhow::bail!("tests.playwright.selectors.wrappers[{index}].export must not be blank");
        }
        let identity = (wrapper.module.as_str(), wrapper.export.as_str());
        if let Some(previous_argument) =
            arguments_by_export.insert(identity, wrapper.test_id_argument)
        {
            if previous_argument != wrapper.test_id_argument {
                anyhow::bail!(
                    "tests.playwright.selectors.wrappers configures `{}:{}` with conflicting testIdArgument values {previous_argument} and {}",
                    wrapper.module,
                    wrapper.export,
                    wrapper.test_id_argument
                );
            }
        }
    }
    Ok(())
}

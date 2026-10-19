use anyhow::Result;
use globset::GlobBuilder;
use std::collections::BTreeMap;

use super::super::schema::PlaywrightSelectorWrapper;

pub(super) fn validate_globs(patterns: &[String], key: &str) -> Result<()> {
    for pattern in patterns {
        GlobBuilder::new(pattern.trim_start_matches("./"))
            .literal_separator(false)
            .build()
            .map_err(|err| anyhow::anyhow!("{key} contains invalid glob `{pattern}`: {err}"))?;
    }
    Ok(())
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

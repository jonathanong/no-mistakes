use anyhow::Result;
use globset::GlobBuilder;
use std::collections::BTreeMap;

use super::super::schema::PlaywrightSelectorWrapper;

#[derive(Debug)]
struct InvalidGlob {
    label: String,
    origin: crate::config::source_locations::ConfigPath,
    pattern: String,
    error: globset::Error,
}

impl std::fmt::Display for InvalidGlob {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "{}: contains invalid glob `{}`: {}",
            self.label, self.pattern, self.error
        )
    }
}

impl std::error::Error for InvalidGlob {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}

pub(super) fn validate_globs(
    patterns: &[String],
    label: &str,
    origin: impl Fn(usize) -> crate::config::source_locations::ConfigPath,
) -> Result<()> {
    for (index, pattern) in patterns.iter().enumerate() {
        GlobBuilder::new(pattern.trim_start_matches("./"))
            .literal_separator(false)
            .build()
            .map_err(|error| InvalidGlob {
                label: format!("{label}[{index}]"),
                origin: origin(index),
                pattern: pattern.clone(),
                error,
            })?;
    }
    Ok(())
}

pub(super) fn annotate(
    error: anyhow::Error,
    source: &str,
    path: &std::path::Path,
) -> anyhow::Error {
    let Some(invalid) = error.downcast_ref::<InvalidGlob>() else {
        return error;
    };
    // The authoritative parser already accepted the syntax. Provenance is
    // best-effort and must never introduce a new configuration rejection.
    let line = crate::config::source_locations::ConfigLocations::parse(source, path)
        .and_then(|locations| locations.line(&invalid.origin))
        .unwrap_or(1);
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

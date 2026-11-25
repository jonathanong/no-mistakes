use super::NoMistakesConfig;
use crate::config::source_locations::ConfigLocations;
use anyhow::Result;
use std::cell::OnceCell;
use std::collections::BTreeMap;
use std::path::Path;

pub(super) fn validate<'a>(
    root: &Path,
    config: &NoMistakesConfig,
    path: &Path,
    documents: impl IntoIterator<Item = (&'a Path, &'a str)>,
) -> Result<()> {
    let documents: BTreeMap<_, _> = documents
        .into_iter()
        .map(|(path, source)| (path, (source, OnceCell::new())))
        .collect();
    crate::codebase::rules::validate_rule_options(root, path, config, &|key| {
        let (file, local) = config
            .config_provenance
            .origin(key)
            .unwrap_or_else(|| (path, key.clone()));
        let (source, locations) = documents.get(file)?;
        let line = locations
            .get_or_init(|| ConfigLocations::parse(source, file))
            .as_ref()?
            .line(&local)?;
        Some((file.to_path_buf(), line))
    })
}

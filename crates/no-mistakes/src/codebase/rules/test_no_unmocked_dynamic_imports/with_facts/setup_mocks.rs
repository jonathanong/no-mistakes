use super::super::{config, resolve_mock_specifiers};
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::ts_resolver::ImportResolver;
use anyhow::Result;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub(super) fn with_facts(
    root: &Path,
    setup_data: &[config::ConfigSetupData],
    test_file: &Path,
    resolver: &ImportResolver<'_>,
    shared: &CheckFactMap,
) -> Result<HashSet<PathBuf>> {
    let mut mocks = HashSet::new();
    let rel_path = crate::codebase::ts_source::relative_slash_path(root, test_file);
    for setup in config::setup_files_for_test_precomputed(&rel_path, setup_data) {
        let Some(file_facts) = shared.ts.get(&setup) else {
            anyhow::bail!("missing shared facts for {}", setup.display());
        };
        if let Some(error) = &file_facts.parse_error {
            anyhow::bail!("failed to parse {}: {error}", setup.display());
        }
        let Some(facts) = file_facts.dynamic_imports.as_ref() else {
            anyhow::bail!("missing dynamic import facts for {}", setup.display());
        };
        mocks.extend(resolve_mock_specifiers(
            &facts.mock_specifiers,
            &setup,
            resolver,
        ));
    }
    Ok(mocks)
}

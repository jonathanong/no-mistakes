use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::ts_source::SourceStore;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(super) fn run(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &Arc<SourceStore>,
    facts: Option<&CheckFactMap>,
) -> Result<Vec<super::super::super::super::RuleFinding>> {
    match facts {
        Some(facts) => {
            crate::codebase::rules::postgres_lock_inventory::check_with_files_sources_and_facts(
                root, config, files, sources, facts,
            )
        }
        None => crate::codebase::rules::postgres_lock_inventory::check_with_files_and_sources(
            root, config, files, sources,
        ),
    }
}

use super::super::super::*;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::ts_source::SourceStore;
use crate::config::v2::NoMistakesConfig;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(super) fn column_naming(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &Arc<SourceStore>,
    facts: Option<&CheckFactMap>,
) -> Result<Vec<RuleFinding>> {
    match facts {
        Some(facts) => postgres_column_naming::check_with_files_sources_and_facts(
            root, config, files, sources, facts,
        ),
        None => postgres_column_naming::check_with_files_and_sources(root, config, files, sources),
    }
}

pub(super) fn finite_text(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &Arc<SourceStore>,
    facts: Option<&CheckFactMap>,
) -> Result<Vec<RuleFinding>> {
    match facts {
        Some(facts) => postgres_finite_text_columns::check_with_files_sources_and_facts(
            root, config, files, sources, facts,
        ),
        None => {
            postgres_finite_text_columns::check_with_files_and_sources(root, config, files, sources)
        }
    }
}

pub(super) fn object_naming(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &Arc<SourceStore>,
    facts: Option<&CheckFactMap>,
) -> Result<Vec<RuleFinding>> {
    match facts {
        Some(facts) => postgres_object_naming::check_with_files_sources_and_facts(
            root, config, files, sources, facts,
        ),
        None => postgres_object_naming::check_with_files_and_sources(root, config, files, sources),
    }
}

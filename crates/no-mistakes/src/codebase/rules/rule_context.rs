use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::ts_source::SourceStore;
use crate::config::v2::NoMistakesConfig;
use std::path::{Path, PathBuf};

/// Borrow the inventory and source store owned by one analysis request.
pub(crate) struct SourcedRuleContext<'a> {
    pub(crate) root: &'a Path,
    pub(crate) config: &'a NoMistakesConfig,
    pub(crate) all_files: &'a [PathBuf],
    pub(crate) sources: &'a SourceStore,
}

impl<'a> SourcedRuleContext<'a> {
    pub(crate) fn new(
        root: &'a Path,
        config: &'a NoMistakesConfig,
        all_files: &'a [PathBuf],
        sources: &'a SourceStore,
    ) -> Self {
        Self {
            root,
            config,
            all_files,
            sources,
        }
    }
}

/// Add the request's prepared fact map for SQL and schema rules.
pub(crate) struct PreparedRuleContext<'a> {
    pub(crate) root: &'a Path,
    pub(crate) config: &'a NoMistakesConfig,
    pub(crate) all_files: &'a [PathBuf],
    pub(crate) sources: &'a SourceStore,
    pub(crate) facts: &'a CheckFactMap,
}

impl<'a> PreparedRuleContext<'a> {
    pub(crate) fn new(
        root: &'a Path,
        config: &'a NoMistakesConfig,
        all_files: &'a [PathBuf],
        sources: &'a SourceStore,
        facts: &'a CheckFactMap,
    ) -> Self {
        Self {
            root,
            config,
            all_files,
            sources,
            facts,
        }
    }
}

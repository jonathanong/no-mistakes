use std::path::{Path, PathBuf};

/// Borrow the inventory and source store owned by one analysis request.
pub struct SourcedRuleContext<'a, Config, Sources> {
    pub root: &'a Path,
    pub config: &'a Config,
    pub all_files: &'a [PathBuf],
    pub sources: &'a Sources,
}

impl<'a, Config, Sources> SourcedRuleContext<'a, Config, Sources> {
    pub fn new(
        root: &'a Path,
        config: &'a Config,
        all_files: &'a [PathBuf],
        sources: &'a Sources,
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
pub struct PreparedRuleContext<'a, Config, Sources, Facts> {
    pub root: &'a Path,
    pub config: &'a Config,
    pub all_files: &'a [PathBuf],
    pub sources: &'a Sources,
    pub facts: &'a Facts,
}

impl<'a, Config, Sources, Facts> PreparedRuleContext<'a, Config, Sources, Facts> {
    pub fn new(
        root: &'a Path,
        config: &'a Config,
        all_files: &'a [PathBuf],
        sources: &'a Sources,
        facts: &'a Facts,
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

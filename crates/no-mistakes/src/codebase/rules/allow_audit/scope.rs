use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use crate::fx::FxHashSet;
use std::path::{Path, PathBuf};

pub(crate) struct CheckedScope {
    checked: FxHashSet<String>,
    inventory: Option<Vec<String>>,
}

impl CheckedScope {
    pub(crate) fn new(root: &Path, sources: Option<&SourceStore>, checked: &[PathBuf]) -> Self {
        Self {
            checked: checked
                .iter()
                .map(|path| relative_slash_path(root, path))
                .collect(),
            inventory: sources.map(|sources| {
                sources
                    .inventory()
                    .paths()
                    .iter()
                    .map(|path| relative_slash_path(root, path))
                    .collect()
            }),
        }
    }

    pub(crate) fn exact(&self, file: &str) -> bool {
        self.checked.contains(file)
    }

    pub(crate) fn glob(&self, pattern: &str) -> bool {
        let Some(inventory) = &self.inventory else {
            return false;
        };
        let Ok(glob) = globset::GlobBuilder::new(pattern.trim_start_matches("./"))
            .literal_separator(false)
            .build()
        else {
            return false;
        };
        let matcher = glob.compile_matcher();
        let mut matched = false;
        for file in inventory.iter().filter(|file| matcher.is_match(file)) {
            if !self.checked.contains(file) {
                return false;
            }
            matched = true;
        }
        matched
    }
}

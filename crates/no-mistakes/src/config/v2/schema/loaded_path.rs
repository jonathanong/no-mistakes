use std::path::{Path, PathBuf};

/// Path selected by the config loader.
///
/// Equality ignores the path so two configs compare as their declared
/// settings. Escape findings still need it: a rule include can drop the
/// config file from the scanned list, and guessing `.no-mistakes.yml` would
/// point `no-mistakes-disable-file` at the wrong file.
#[derive(Clone, Debug, Default)]
pub struct LoadedConfigPath(pub Option<PathBuf>);

impl PartialEq for LoadedConfigPath {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl LoadedConfigPath {
    pub(crate) fn relative_to(&self, root: &Path) -> Option<String> {
        self.0
            .as_deref()
            .map(|path| crate::codebase::ts_source::relative_slash_path(root, path))
    }
}

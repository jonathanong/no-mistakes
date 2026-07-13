use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct PlaywrightConfig {
    pub name: Option<String>,
    pub projects: Vec<TestProject>,
}

#[derive(Clone)]
pub struct TestProject {
    pub name: Option<String>,
    pub config_dir: PathBuf,
    pub test_dir: String,
    pub test_match: Vec<String>,
    pub test_ignore: Vec<String>,
    pub base_url: Option<String>,
    /// The `testIdAttribute` resolved from the Playwright config's `use` block,
    /// or `None` when it could not be read statically (e.g. the config is built
    /// by a helper function). `None` is the signal for callers to fall back to
    /// the configured `tests.playwright.selectors.testIds`.
    pub test_id_attribute: Option<String>,
}

#[derive(Default)]
pub(super) struct ParsedOptions {
    pub(super) name: Option<String>,
    pub(super) test_dir: Option<String>,
    pub(super) test_match: Option<Vec<String>>,
    pub(super) test_ignore: Option<Vec<String>>,
    pub(super) base_url: Option<String>,
    pub(super) test_id_attribute: Option<String>,
}

impl PlaywrightConfig {
    pub fn test_id_attributes(&self) -> Vec<String> {
        let mut attributes: Vec<String> = self
            .projects
            .iter()
            .filter_map(|project| project.test_id_attribute.clone())
            .collect();
        attributes.sort();
        attributes.dedup();
        attributes
    }
}

impl TestProject {
    pub fn test_dir(&self, root: &Path) -> PathBuf {
        let path = Path::new(&self.test_dir);
        if path.is_absolute() {
            path.to_path_buf()
        } else if self.config_dir.is_absolute() {
            self.config_dir.join(path)
        } else {
            root.join(&self.config_dir).join(path)
        }
    }
}

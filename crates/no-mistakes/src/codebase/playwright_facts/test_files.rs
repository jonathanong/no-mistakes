use std::path::PathBuf;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct TestProjectContext {
    pub(crate) base_url: Option<String>,
    /// The attribute(s) that `getByTestId(...)` resolves to for this project's
    /// tests. Usually one, but the `selectors.testIds` fallback (used when the
    /// Playwright config's `testIdAttribute` is not statically readable) can
    /// contribute several.
    pub(crate) test_id_attributes: Vec<String>,
}

#[derive(Clone)]
pub struct DiscoveredTestFile {
    pub(crate) path: PathBuf,
    pub(crate) contexts: Vec<TestProjectContext>,
}

impl DiscoveredTestFile {
    pub(crate) fn base_urls(&self) -> Vec<String> {
        let mut urls: Vec<String> = self
            .contexts
            .iter()
            .filter_map(|context| context.base_url.clone())
            .collect();
        urls.sort();
        urls.dedup();
        urls
    }

    pub(crate) fn test_id_attributes(&self) -> Vec<String> {
        let mut attributes: Vec<String> = self
            .contexts
            .iter()
            .flat_map(|context| context.test_id_attributes.iter().cloned())
            .collect();
        attributes.sort();
        attributes.dedup();
        attributes
    }
}

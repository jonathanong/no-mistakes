use crate::fx::FxHashMap;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct ImportRef {
    pub(crate) specifier: Arc<str>,
    pub(crate) name: Arc<str>,
}

impl ImportRef {
    pub(crate) fn new(specifier: &str, name: &str) -> Self {
        Self {
            specifier: Arc::from(specifier),
            name: Arc::from(name),
        }
    }

    pub(crate) fn is_playwright_test(&self) -> bool {
        self.specifier.as_ref() == "@playwright/test" && self.name.as_ref() == "test"
    }
}

pub(crate) struct FixtureInitializer {
    pub(crate) import: ImportRef,
    pub(crate) derived_at: Option<u32>,
}

#[derive(Default)]
pub(crate) struct FixtureInitializers(pub(crate) FxHashMap<String, FixtureInitializer>);

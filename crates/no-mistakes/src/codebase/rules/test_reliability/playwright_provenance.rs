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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct WrapperCall {
    pub(crate) callee: ImportRef,
    pub(crate) arg_index: usize,
}

#[derive(Clone, Debug)]
pub(crate) struct WrapperSummary {
    pub(crate) param_index: usize,
    pub(crate) wrappers: Vec<WrapperCall>,
    pub(crate) derived_at: Option<u32>,
}

#[derive(Clone, Debug)]
pub(crate) struct FixtureInitializer {
    pub(crate) import: ImportRef,
    pub(crate) wrappers: Vec<WrapperCall>,
    pub(crate) derived_at: Option<u32>,
}

#[derive(Default)]
pub(crate) struct FixtureInitializers {
    pub(crate) values: FxHashMap<String, FixtureInitializer>,
    pub(crate) uncertain: FxHashMap<String, (&'static str, Vec<ImportRef>)>,
    pub(crate) wrappers: FxHashMap<String, WrapperSummary>,
}

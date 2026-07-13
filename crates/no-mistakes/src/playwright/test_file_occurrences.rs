use crate::playwright::analysis::text_types::PlaywrightTextLocator;
use crate::playwright::playwright_tests::TestOccurrence;
use crate::playwright::selectors::{PlaywrightHelperReference, PlaywrightSelector};
use std::sync::Arc;

#[derive(Default)]
pub(crate) struct CommonOccurrences {
    pub(crate) text_locators: Vec<TestOccurrence<PlaywrightTextLocator>>,
    pub(crate) helper_references: Vec<TestOccurrence<PlaywrightHelperReference>>,
}

#[derive(Default)]
pub(crate) struct VariantOccurrences {
    pub(crate) urls: Vec<TestOccurrence<String>>,
    pub(crate) selectors: Vec<TestOccurrence<PlaywrightSelector>>,
}

#[derive(Clone, Default)]
pub(crate) struct TestFileOccurrences {
    pub(crate) common: Arc<CommonOccurrences>,
    pub(crate) variant: Arc<VariantOccurrences>,
}

impl TestFileOccurrences {
    pub(crate) fn urls(&self) -> &[TestOccurrence<String>] {
        &self.variant.urls
    }

    pub(crate) fn selectors(&self) -> &[TestOccurrence<PlaywrightSelector>] {
        &self.variant.selectors
    }

    pub(crate) fn text_locators(&self) -> &[TestOccurrence<PlaywrightTextLocator>] {
        &self.common.text_locators
    }

    pub(crate) fn helper_references(&self) -> &[TestOccurrence<PlaywrightHelperReference>] {
        &self.common.helper_references
    }
}

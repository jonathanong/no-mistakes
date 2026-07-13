use super::PlaywrightOccurrenceKey;
use crate::playwright::test_file_occurrences::{
    CommonOccurrences, TestFileOccurrences, VariantOccurrences,
};
use std::collections::BTreeMap;
use std::sync::Arc;

pub(crate) struct PlaywrightTestFacts {
    common: Arc<CommonOccurrences>,
    variants: BTreeMap<PlaywrightOccurrenceKey, Arc<VariantOccurrences>>,
}

impl PlaywrightTestFacts {
    pub(crate) fn new(
        common: Arc<CommonOccurrences>,
        variants: BTreeMap<PlaywrightOccurrenceKey, Arc<VariantOccurrences>>,
    ) -> Self {
        Self { common, variants }
    }

    pub(crate) fn select(&self, key: &PlaywrightOccurrenceKey) -> Option<TestFileOccurrences> {
        self.variants.get(key).map(|variant| TestFileOccurrences {
            common: Arc::clone(&self.common),
            variant: Arc::clone(variant),
        })
    }

    pub(crate) fn common(&self) -> &CommonOccurrences {
        &self.common
    }
}

#[cfg(test)]
mod tests;

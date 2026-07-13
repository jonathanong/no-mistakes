use super::{Arc, BTreeMap, CommonOccurrences, PlaywrightTestFacts, TestFileOccurrences};

impl PlaywrightTestFacts {
    pub(crate) fn empty() -> Self {
        Self::new(Arc::new(CommonOccurrences::default()), BTreeMap::new())
    }

    pub(crate) fn all(&self) -> Vec<TestFileOccurrences> {
        self.variants
            .values()
            .map(|variant| TestFileOccurrences {
                common: Arc::clone(&self.common),
                variant: Arc::clone(variant),
            })
            .collect()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum TestStatus {
    Active,
    Conditional,
    Skipped,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct TestPolicy {
    pub assert_conditional_tests: bool,
    pub allow_skipped_tests: bool,
}

impl TestStatus {
    pub fn merge(self, other: Self) -> Self {
        if self.rank() >= other.rank() {
            self
        } else {
            other
        }
    }

    fn rank(self) -> u8 {
        match self {
            TestStatus::Active => 0,
            TestStatus::Conditional => 1,
            TestStatus::Skipped => 2,
        }
    }
}

impl TestPolicy {
    pub fn allows(self, status: TestStatus) -> bool {
        match status {
            TestStatus::Active => true,
            TestStatus::Conditional => !self.assert_conditional_tests,
            TestStatus::Skipped => self.allow_skipped_tests,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum TestOccurrenceScope {
    File,
    Hook,
    TeardownHook,
    Test,
}

impl TestOccurrenceScope {
    #[cfg_attr(test, inline(never))]
    pub(crate) fn is_runnable(self) -> bool {
        matches!(self, Self::Hook | Self::Test)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct TestOccurrence<T> {
    pub value: T,
    pub status: TestStatus,
    pub scope: TestOccurrenceScope,
    pub test_name: Option<String>,
    pub describe_path: Vec<String>,
    pub line: u32,
}

impl<T: PartialEq> TestOccurrence<T> {
    fn same_identity(&self, other: &Self) -> bool {
        self.value == other.value
            && self.status == other.status
            && self.scope == other.scope
            && self.test_name == other.test_name
            && self.describe_path == other.describe_path
            && self.line == other.line
    }
}

pub(crate) fn dedup_occurrences_by_identity<T: Ord + PartialEq>(
    occurrences: &mut Vec<TestOccurrence<T>>,
) {
    occurrences.sort();
    occurrences.dedup_by(|left, right| left.same_identity(right));
}

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod tests;

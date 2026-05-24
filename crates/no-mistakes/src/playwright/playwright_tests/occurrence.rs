use super::TestStatus;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct TestOccurrence<T> {
    pub value: T,
    pub status: TestStatus,
    pub test_name: Option<String>,
    pub describe_path: Vec<String>,
    pub line: u32,
}

impl<T: PartialEq> TestOccurrence<T> {
    fn same_identity_ignoring_line(&self, other: &Self) -> bool {
        self.value == other.value
            && self.status == other.status
            && self.test_name == other.test_name
            && self.describe_path == other.describe_path
    }
}

pub(crate) fn dedup_occurrences_by_identity<T: Ord + PartialEq>(
    occurrences: &mut Vec<TestOccurrence<T>>,
) {
    occurrences.sort();
    occurrences.dedup_by(|left, right| left.same_identity_ignoring_line(right));
}

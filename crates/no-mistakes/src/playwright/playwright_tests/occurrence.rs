use super::TestStatus;

#[derive(Clone, Debug)]
pub struct TestOccurrence<T> {
    pub value: T,
    pub status: TestStatus,
    pub test_name: Option<String>,
    pub describe_path: Vec<String>,
    pub line: u32,
}

impl<T: PartialEq> PartialEq for TestOccurrence<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
            && self.status == other.status
            && self.test_name == other.test_name
            && self.describe_path == other.describe_path
    }
}

impl<T: Eq> Eq for TestOccurrence<T> {}

impl<T: Ord> Ord for TestOccurrence<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (
            &self.value,
            self.status,
            &self.test_name,
            &self.describe_path,
        )
            .cmp(&(
                &other.value,
                other.status,
                &other.test_name,
                &other.describe_path,
            ))
    }
}

impl<T: Ord> PartialOrd for TestOccurrence<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

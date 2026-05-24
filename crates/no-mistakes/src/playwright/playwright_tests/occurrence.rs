use super::TestStatus;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct TestOccurrence<T> {
    pub value: T,
    pub status: TestStatus,
    pub test_name: Option<String>,
    pub describe_path: Vec<String>,
    pub line: u32,
}

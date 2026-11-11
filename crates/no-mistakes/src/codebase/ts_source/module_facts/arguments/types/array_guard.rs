use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ArrayGuard {
    pub(crate) owner: u32,
    pub(crate) parameter: usize,
    pub(crate) index: usize,
    pub(crate) length: bool,
    pub(crate) expected: Value,
}

use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArrayGuard {
    pub owner: u32,
    pub parameter: usize,
    pub index: usize,
    pub length: bool,
    pub expected: Value,
}

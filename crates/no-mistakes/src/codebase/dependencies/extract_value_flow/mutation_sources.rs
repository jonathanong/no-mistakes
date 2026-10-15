use super::*;

// None is a dynamic segment; Some("*") is an ordinary literal property.
pub type ValueMutationPath = Vec<Option<String>>;
pub type ValueReceiverMutations =
    FxHashMap<usize, FxHashMap<String, std::sync::Arc<mutation_history::ValueMutationHistory>>>;
pub type ValueDefaultMutations = FxHashMap<usize, Vec<ValueDefaultMutationSource>>;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueDefaultMutationSource {
    pub path: ValueMutationPath,
    pub history: std::sync::Arc<mutation_history::ValueMutationHistory>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueMutationInfo {
    pub offset: u32,
    pub contexts: FxHashSet<Option<CallableId>>,
}

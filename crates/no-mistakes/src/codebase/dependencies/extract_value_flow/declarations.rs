pub(super) type DefaultOriginIndex =
    FxHashMap<usize, FxHashMap<String, Vec<default_origins::ReceiverOrigin>>>;
pub(super) type PriorMemberMutations = receiver_histories::ReceiverHistories;
pub(super) type ReceiverBindingOrigins = mutable_origins::ReceiverBindingOrigins;
pub(super) type ForeignReceiverWrites = mutable_origins::ForeignReceiverWrites;
pub(super) type ReferenceLinks = FxHashMap<usize, FxHashMap<String, Vec<(usize, String)>>>;

/// Small syntactic annotation shapes; resolution uses the prepared fact catalog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueType {
    Reference(String),
    Member(Box<ValueType>, String),
    Object(FxHashMap<String, ValueType>),
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallableAssignment {
    pub scope: usize,
    pub name: String,
    pub completed_at: u32,
    pub callable: Option<CallableId>,
    pub owner: Option<CallableId>,
    pub region_start: Option<u32>,
    pub guaranteed: bool,
}

pub(crate) use module_provider_guards::ModuleAssignedMutations;
pub(crate) use module_provider_intervals::ModuleProviderTarget;
pub(crate) use mutation_history::ValueAssignedMutations;
pub use mutation_history::ValueMutationHistory;
pub use mutation_sources::{
    ValueDefaultMutationSource, ValueDefaultMutations, ValueMutationInfo, ValueMutationPath,
    ValueReceiverMutations,
};
pub use replacements::{
    ValueRepeatCall, ValueRepeatRegion, ValueReplacementFacts, ValueReplacementWrite,
};
pub use value_descriptors::{
    ValueBinding, ValueCall, ValueExpression, ValueParameter, ValueProjection,
};

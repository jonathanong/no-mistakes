use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueExpression {
    Named {
        name: String,
        scope: Option<usize>,
        offset: u32,
        context: Option<CallableId>,
    },
    Call(u32, u32),
    Awaited(std::sync::Arc<ValueExpression>),
    Callable(CallableId),
    Super(CallableId),
    Member(std::sync::Arc<ValueExpression>, String),
    Object(std::sync::Arc<FxHashMap<String, ValueExpression>>, bool),
    Array(std::sync::Arc<FxHashMap<String, ValueExpression>>),
    Undefined,
    Null,
    Nullish(
        std::sync::Arc<ValueExpression>,
        std::sync::Arc<ValueExpression>,
    ),
    Default(
        std::sync::Arc<ValueExpression>,
        std::sync::Arc<ValueExpression>,
    ),
    /// Both arms of a conditional array element. Flow values stay Unknown.
    Either(
        std::sync::Arc<ValueExpression>,
        std::sync::Arc<ValueExpression>,
    ),
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueBinding {
    /// Declaring callable; distinct from a nested closure reading this binding.
    pub owner: Option<CallableId>,
    pub name: String,
    pub scope: usize,
    pub declared_at: u32,
    /// The initializer is reacquired within this callable execution scope.
    pub repeated: bool,
    pub invalidated_at: Option<u32>,
    pub mutation_contexts: FxHashSet<Option<CallableId>>,
    pub value: ValueExpression,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueProjection {
    Member(String),
    Element(String),
    Default(ValueExpression, usize),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueParameter {
    pub owner: CallableId,
    pub index: usize,
    pub name: String,
    pub scope: usize,
    pub default: ValueExpression,
    pub projection: Vec<ValueProjection>,
    pub initialization_order: usize,
    pub default_order: usize,
    pub default_mutations: Option<std::sync::Arc<ValueDefaultMutations>>,
    pub invalidated_at: Option<u32>,
    pub mutation_contexts: FxHashSet<Option<CallableId>>,
    /// An explicit reference in the annotation namespace, distinct from runtime names.
    pub type_reference: Option<ValueType>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueCall {
    pub offset: u32,
    pub end: u32,
    pub source_callee: String,
    pub context: Option<CallableId>,
    pub callee: ValueExpression,
    pub arguments: std::sync::Arc<Vec<ValueExpression>>,
    pub bound: bool,
    pub spread_at: Option<usize>,
    /// A direct, module-level arrow invocation outside a repeated construct.
    pub once: bool,
    /// Repetition within this execution scope; qualified callable bodies do not
    /// inherit repetition solely from their creation site.
    pub repeated: bool,
}

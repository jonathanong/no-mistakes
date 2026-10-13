use super::*;

/// Syntactic value flow retained by the import collector. Resolution uses the
/// prepared catalog and never reparses a factory or helper body.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ValueFlowFacts {
    #[cfg(feature = "test-instrumentation")]
    pub reference_link_constructions: usize,
    pub bindings: Vec<ValueBinding>,
    pub parameters: Vec<ValueParameter>,
    pub returns: Vec<(CallableId, ValueExpression)>,
    pub calls: Vec<ValueCall>,
    pub async_callables: FxHashSet<CallableId>,
    pub generator_callables: FxHashSet<CallableId>,
}
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
    Object(std::sync::Arc<FxHashMap<String, ValueExpression>>),
    Array(std::sync::Arc<FxHashMap<String, ValueExpression>>),
    Undefined,
    Default(
        std::sync::Arc<ValueExpression>,
        std::sync::Arc<ValueExpression>,
    ),
    Unknown,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueBinding {
    pub name: String,
    pub scope: usize,
    pub declared_at: u32,
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
    pub invalidated_at: Option<u32>,
    pub mutation_contexts: FxHashSet<Option<CallableId>>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueCall {
    pub offset: u32,
    pub end: u32,
    pub source_callee: String,
    pub callee: ValueExpression,
    pub arguments: std::sync::Arc<Vec<ValueExpression>>,
    pub bound: bool,
    pub spread_at: Option<usize>,
}
#[path = "extract_value_flow/collect.rs"]
mod collect;

#[path = "extract_value_flow/expressions.rs"]
mod expressions;

#[path = "extract_value_flow/adapters.rs"]
mod adapters;
#[path = "extract_value_flow/completion.rs"]
mod completion;
#[path = "extract_value_flow/mutations.rs"]
mod mutations;

#[path = "extract_value_flow/result_kind.rs"]
mod result_kind;

#[path = "extract_value_flow/default_export.rs"]
mod default_export;
#[path = "extract_value_flow/templates.rs"]
mod templates;

#[path = "extract_value_flow/nested.rs"]
mod nested;

#[path = "extract_value_flow/parameter_patterns.rs"]
mod parameter_patterns;

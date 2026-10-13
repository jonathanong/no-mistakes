use super::*;

/// Syntactic value flow retained by the import collector. Resolution uses the
/// prepared catalog and never reparses a factory or helper body.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ValueFlowFacts {
    pub bindings: Vec<ValueBinding>,
    pub parameters: Vec<ValueParameter>,
    pub returns: Vec<(CallableId, ValueExpression)>,
    pub calls: Vec<ValueCall>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueExpression {
    Named {
        name: String,
        scope: Option<usize>,
        offset: u32,
    },
    Call(u32, u32),
    Callable(CallableId),
    Member(std::sync::Arc<ValueExpression>, String),
    Object(Vec<(String, ValueExpression)>),
    Unknown,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueBinding {
    pub name: String,
    pub scope: usize,
    pub declared_at: u32,
    pub invalidated_at: Option<u32>,
    pub value: ValueExpression,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueParameter {
    pub owner: CallableId,
    pub index: usize,
    pub name: String,
    pub scope: usize,
    pub default: ValueExpression,
    pub invalidated_at: Option<u32>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueCall {
    pub offset: u32,
    pub end: u32,
    pub source_callee: String,
    pub callee: ValueExpression,
    pub arguments: Vec<ValueExpression>,
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

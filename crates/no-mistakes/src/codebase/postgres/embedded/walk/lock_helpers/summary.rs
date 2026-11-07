use super::*;
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct Summary {
    pub(crate) calls: Vec<Query>,
}
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct Query {
    pub(crate) span: (u32, u32),
    pub(crate) access: ExecutorAccess,
    pub(crate) executor: usize,
    pub(crate) sql: String,
    pub(crate) line: u32,
    pub(crate) parameters: Vec<Parameter>,
}
#[derive(Clone, PartialEq, Eq)]
pub(crate) enum Parameter {
    Argument(usize),
    Literal(String),
    Unknown,
}
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct Invocation {
    pub(crate) accesses: Vec<Option<ExecutorAccess>>,
    pub(crate) resolve_helper: bool,
    pub(crate) name: String,
    pub(crate) executor: usize,
    pub(crate) arguments: Vec<Option<String>>,
    pub(crate) call: EmbeddedSqlCall,
    pub(crate) span: (u32, u32),
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExecutorAccess {
    Callable,
    Object,
}
pub(super) fn access(expression: &Expression<'_>) -> Option<ExecutorAccess> {
    match unwrap_ts_wrappers(expression) {
        Expression::Identifier(_) => Some(ExecutorAccess::Object),
        Expression::StaticMemberExpression(member)
            if member.property.name == "query" && !member.optional =>
        {
            Some(ExecutorAccess::Callable)
        }
        _ => None,
    }
}
pub(super) fn summaries(helpers: &LockHelpers, _source: &str) -> FxHashMap<String, Summary> {
    helpers
        .iter()
        .map(|(name, helper)| {
            let calls = helper.calls.clone();
            (name.clone(), Summary { calls })
        })
        .collect()
}

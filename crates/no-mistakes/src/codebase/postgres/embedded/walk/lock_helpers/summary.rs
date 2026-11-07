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
pub(super) fn summaries(helpers: &LockHelpers<'_>, source: &str) -> FxHashMap<String, Summary> {
    helpers
        .iter()
        .map(|(name, helper)| {
            let calls = helper
                .calls
                .iter()
                .map(|(executor, access, query)| {
                    let parameters = match query
                        .arguments
                        .get(1)
                        .and_then(|arg| arg.as_expression())
                        .map(unwrap_ts_wrappers)
                    {
                        Some(Expression::ArrayExpression(array)) => array
                            .elements
                            .iter()
                            .map(|item| {
                                let Some(expression) = item.as_expression() else {
                                    return Parameter::Unknown;
                                };
                                if let Expression::Identifier(id) = unwrap_ts_wrappers(expression) {
                                    return helper
                                        .params
                                        .iter()
                                        .position(|param| *param == id.name.as_str())
                                        .map(Parameter::Argument)
                                        .unwrap_or(Parameter::Unknown);
                                }
                                super::super::transaction::literal_identity(expression)
                                    .map(Parameter::Literal)
                                    .unwrap_or(Parameter::Unknown)
                            })
                            .collect(),
                        _ => Vec::new(),
                    };
                    Query {
                        span: (query.span.start, query.span.end),
                        access: *access,
                        executor: *executor,
                        sql: crate::codebase::postgres::embedded::sql_text(
                            query.arguments[0].as_expression().unwrap(),
                        )
                        .unwrap(),
                        line: byte_offset_to_line(source, query.span.start as usize),
                        parameters,
                    }
                })
                .collect();
            (name.clone(), Summary { calls })
        })
        .collect()
}

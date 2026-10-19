use sqlparser::ast::Statement;
use sqlparser::tokenizer::TokenWithSpan;
use std::sync::Arc;

/// Recovered SQL borrows the tokenized fragment that produced its AST spans.
#[derive(Debug)]
pub(crate) struct LocatedStatement {
    pub(crate) statement: Statement,
    pub(crate) source: Option<Arc<[TokenWithSpan]>>,
    /// Recovered procedural expressions supply calls without changing legacy
    /// statement, query, or lifecycle projections.
    pub(crate) function_projection: bool,
    /// Only the recovery wrapper projection is synthetic; nested SQL stays scoped.
    pub(crate) synthetic_select: bool,
    pub(crate) recovered_functions: Vec<crate::codebase::postgres::SqlFunctionCallFact>,
    pub(crate) policy_arbiter: Option<crate::codebase::postgres::source::PolicyArbiter>,
    /// The statement is the entire semicolon-delimited chunk, not a recovery from a failed chunk.
    pub(crate) complete: bool,
    /// Tokens of the semicolon chunk this statement was recovered from.
    /// Shared by every statement that came from that chunk, including a marker
    /// left behind when the chunk produced no statement.
    pub(crate) chunk_tokens: Option<Arc<[TokenWithSpan]>>,
}

impl LocatedStatement {
    pub(super) fn plain(statement: Statement) -> Self {
        Self {
            statement,
            source: None,
            function_projection: false,
            synthetic_select: false,
            recovered_functions: Vec::new(),
            policy_arbiter: None,
            complete: false,
            chunk_tokens: None,
        }
    }

    pub(super) fn functions(statement: Statement, synthetic_select: bool) -> Self {
        Self {
            function_projection: true,
            synthetic_select,
            ..Self::plain(statement)
        }
    }
}

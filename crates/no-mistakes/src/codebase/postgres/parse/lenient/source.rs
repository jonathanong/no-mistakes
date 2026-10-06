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
}

impl LocatedStatement {
    pub(super) fn plain(statement: Statement) -> Self {
        Self {
            statement,
            source: None,
            function_projection: false,
        }
    }

    pub(super) fn functions(statement: Statement) -> Self {
        Self {
            function_projection: true,
            ..Self::plain(statement)
        }
    }
}

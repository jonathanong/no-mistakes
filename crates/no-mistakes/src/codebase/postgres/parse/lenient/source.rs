use sqlparser::ast::Statement;
use sqlparser::tokenizer::TokenWithSpan;
use std::sync::Arc;

/// Recovered SQL borrows the tokenized fragment that produced its AST spans.
#[derive(Debug)]
pub(crate) struct LocatedStatement {
    pub(crate) statement: Statement,
    pub(crate) source: Option<Arc<[TokenWithSpan]>>,
}

impl LocatedStatement {
    pub(super) fn plain(statement: Statement) -> Self {
        Self {
            statement,
            source: None,
        }
    }
}

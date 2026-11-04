use super::{SqlBoundFact, SqlBoundQuery};
use sqlparser::ast::Statement;
use std::collections::BTreeSet;

/// Prepared, source-ordered lifecycle inputs for catalog-dependent SQL bounds.
/// The SQL AST and view reads are collected once; each catalog only projects them.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[doc(hidden)]
pub struct SqlLifecycleFacts {
    pub(crate) raw_bounds: Vec<SqlBoundFact>,
    pub(crate) batches: Vec<SqlLifecycleBatch>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SqlLifecycleBatch {
    pub source: Statement,
    pub steps: Vec<SqlLifecycleStep>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SqlLifecycleStep {
    pub statement: Statement,
    pub first_bound: usize,
    pub last_bound: usize,
    pub view_reads: Option<SqlViewReads>,
    pub function_bounds: Option<SqlFunctionBounds>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SqlViewReads {
    pub query: SqlBoundQuery,
    pub names: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SqlFunctionBounds {
    pub positions: Option<Vec<(u32, u32)>>,
    pub(in crate::codebase::postgres) table_tokens:
        Option<super::super::statements::TableTokenCursor>,
}

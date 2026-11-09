use sqlparser::ast::Query;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SqlGroupingQuery {
    pub(crate) query: Box<Query>,
    pub(crate) sites: Vec<(usize, usize)>,
    pub(crate) parameter_sites: Arc<[(u32, u32)]>,
}

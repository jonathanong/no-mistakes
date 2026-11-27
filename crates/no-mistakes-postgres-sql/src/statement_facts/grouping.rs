use sqlparser::ast::Query;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlGroupingQuery {
    pub query: Arc<Query>,
    pub sites: Vec<(usize, usize)>,
    pub parameter_sites: Arc<[(u32, u32)]>,
}

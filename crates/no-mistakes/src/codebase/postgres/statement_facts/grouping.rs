use sqlparser::ast::Query;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SqlGroupingQuery {
    pub(crate) query: Box<Query>,
    pub(crate) sites: Vec<(usize, usize)>,
}

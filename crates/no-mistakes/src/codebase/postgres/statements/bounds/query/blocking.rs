use crate::fx::{fx_map, FxHashMap};
use sqlparser::ast::SetExpr;

pub(super) struct BlockingStatuses {
    // Keys point into the borrowed query AST; this map is dropped before bound_body returns.
    pub(super) by_set: FxHashMap<*const SetExpr, bool>,
    pub(super) visits: usize,
}

impl BlockingStatuses {
    pub(super) fn new() -> Self {
        Self {
            by_set: fx_map(),
            visits: 0,
        }
    }

    pub(super) fn complete(&mut self, set: &SetExpr) -> bool {
        self.visits += 1;
        self.by_set[&(set as *const SetExpr)]
    }
}

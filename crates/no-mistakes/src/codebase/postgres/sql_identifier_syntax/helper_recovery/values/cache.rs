use super::super::budget::Cost;
use super::{Execution, Key, Value};
use crate::fx::FxHashMap;
use std::{cell::RefCell, collections::BTreeSet, path::PathBuf, rc::Rc};
#[cfg(test)]
#[path = "cache_tests.rs"]
mod cache_tests;

#[derive(Clone)]
pub(super) struct Summary {
    pub(super) args: Vec<Value>,
    pub(super) outcomes: Vec<Value>,
    pub(super) cost: Cost,
    pub(super) executed: Vec<Execution>,
    pub(super) failed_sources: BTreeSet<PathBuf>,
}

#[derive(Default)]
pub(in crate::codebase::postgres::sql_identifier_syntax::helper_recovery) struct Cache {
    pub(super) complete: FxHashMap<(Key, u64), Vec<Summary>>,
    pub(in crate::codebase::postgres::sql_identifier_syntax::helper_recovery) observed_sources:
        Rc<RefCell<Vec<PathBuf>>>,
}

impl Cache {
    pub(in crate::codebase::postgres::sql_identifier_syntax::helper_recovery) fn take_observed_sources(
        &mut self,
    ) -> BTreeSet<PathBuf> {
        std::mem::take(&mut *self.observed_sources.borrow_mut())
            .into_iter()
            .collect()
    }
}

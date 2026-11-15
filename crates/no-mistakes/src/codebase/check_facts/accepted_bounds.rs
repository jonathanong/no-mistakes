use serde::Serialize;
use std::sync::{Arc, Mutex};

use super::map::CheckFactMap;

/// A reviewed per-statement row limit that suppressed one rule's findings.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceptedStatementBound {
    pub rule: String,
    pub file: String,
    pub line: usize,
    pub rows: u64,
    pub justification: String,
}

impl CheckFactMap {
    pub(crate) fn record_accepted_statement_bound(&self, bound: AcceptedStatementBound) {
        self.accepted_statement_bounds
            .lock()
            .expect("accepted statement bounds lock")
            .push(bound);
    }

    pub fn accepted_statement_bounds(&self) -> Vec<AcceptedStatementBound> {
        let mut bounds = self
            .accepted_statement_bounds
            .lock()
            .expect("accepted statement bounds lock")
            .drain(..)
            .collect::<Vec<_>>();
        bounds.sort();
        bounds.dedup();
        bounds
    }
}

pub(crate) fn empty_sink() -> Arc<Mutex<Vec<AcceptedStatementBound>>> {
    Arc::default()
}

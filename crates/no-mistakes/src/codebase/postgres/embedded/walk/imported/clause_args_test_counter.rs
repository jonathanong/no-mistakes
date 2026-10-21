//! Thread-local evidence of calls entering the actual classifier boundary.
use std::cell::RefCell;

thread_local! {
    static CLASSIFIED_CALLS: RefCell<Vec<u32>> = const { RefCell::new(Vec::new()) };
}

pub(super) fn record(start: u32) {
    CLASSIFIED_CALLS.with_borrow_mut(|calls| calls.push(start));
}

pub(in crate::codebase::postgres::embedded::walk) fn reset() {
    CLASSIFIED_CALLS.with_borrow_mut(Vec::clear);
}

pub(in crate::codebase::postgres::embedded::walk) fn take() -> Vec<u32> {
    CLASSIFIED_CALLS.with_borrow_mut(std::mem::take)
}

macro_rules! record_clause_args {
    ($start:expr) => {
        clause_args_test_counter::record($start);
    };
}

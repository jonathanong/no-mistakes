use sqlparser::ast::{Query, Visit, Visitor};
use std::collections::HashSet;
use std::ops::ControlFlow;

pub(super) fn complete(query: &Query, visited: &HashSet<*const Query>) -> bool {
    struct Complete<'a>(&'a HashSet<*const Query>);

    impl Visitor for Complete<'_> {
        type Break = ();

        fn pre_visit_query(&mut self, query: &Query) -> ControlFlow<()> {
            if self.0.contains(&std::ptr::from_ref(query)) {
                ControlFlow::Continue(())
            } else {
                ControlFlow::Break(())
            }
        }
    }

    query.visit(&mut Complete(visited)).is_continue()
}

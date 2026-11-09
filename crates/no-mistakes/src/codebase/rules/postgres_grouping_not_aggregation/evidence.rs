use super::{scope::CatalogScope, walk};
use sqlparser::ast::*;
use std::ops::ControlFlow;
pub(super) fn catalogued(node: &impl Visit, scope: &CatalogScope<'_>) -> bool {
    struct Known<'a, 'b> {
        scope: &'a CatalogScope<'b>,
        count: usize,
    }
    impl Visitor for Known<'_, '_> {
        type Break = ();
        fn pre_visit_table_factor(&mut self, factor: &TableFactor) -> ControlFlow<()> {
            if let Some(relation) = walk::relation(factor) {
                if self.scope.relation(&relation.name).is_none() {
                    return ControlFlow::Break(());
                }
                self.count += 1;
            }
            ControlFlow::Continue(())
        }
    }
    let mut known = Known { scope, count: 0 };
    node.visit(&mut known).is_continue() && known.count > 0
}

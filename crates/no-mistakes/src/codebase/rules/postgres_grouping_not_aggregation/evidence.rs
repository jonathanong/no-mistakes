use super::{
    scope::{definitions, CatalogScope},
    walk,
};
use sqlparser::ast::*;
use std::ops::ControlFlow;
pub(super) fn catalogued(node: &impl Visit, scope: &CatalogScope<'_>) -> bool {
    struct Known<'a, 'b> {
        scope: &'a CatalogScope<'b>,
        count: usize,
        shadowed: Vec<std::collections::BTreeSet<String>>,
        definitions: definitions::Names,
    }
    impl Visitor for Known<'_, '_> {
        type Break = ();
        fn pre_visit_query(&mut self, query: &Query) -> ControlFlow<()> {
            let body = definitions::enter(
                query,
                Some(self.shadowed.last().unwrap_or(self.scope.shadowed)),
                &mut self.definitions,
            );
            self.shadowed.push(body);
            ControlFlow::Continue(())
        }
        fn post_visit_query(&mut self, _: &Query) -> ControlFlow<()> {
            self.shadowed.pop();
            ControlFlow::Continue(())
        }
        fn pre_visit_table_factor(&mut self, factor: &TableFactor) -> ControlFlow<()> {
            if let Some(relation) = walk::relation(factor) {
                let scope = CatalogScope {
                    catalog: self.scope.catalog,
                    parameters: self.scope.parameters,
                    shadowed: self.shadowed.last().unwrap_or(self.scope.shadowed),
                };
                if scope.relation(&relation.name).is_none() {
                    return ControlFlow::Break(());
                }
                self.count += 1;
            }
            ControlFlow::Continue(())
        }
    }
    let mut known = Known {
        scope,
        count: 0,
        shadowed: Vec::new(),
        definitions: Default::default(),
    };
    node.visit(&mut known).is_continue() && known.count > 0
}

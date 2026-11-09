use super::super::{expressions::ident, scope::definitions, walk};
use sqlparser::ast::{Cte, Query, TableFactor, Visit, Visitor};
use std::{
    collections::{BTreeMap, BTreeSet},
    ops::ControlFlow,
};

pub(super) fn factor_key(factor: &TableFactor) -> usize {
    std::ptr::from_ref(factor) as usize
}

pub(super) fn cte(query: &Query, target: &Cte) -> BTreeSet<usize> {
    struct References {
        name: String,
        target: usize,
        visible: Vec<bool>,
        definitions: BTreeMap<usize, bool>,
        factors: BTreeSet<usize>,
    }
    impl Visitor for References {
        type Break = ();
        fn pre_visit_query(&mut self, query: &Query) -> ControlFlow<()> {
            let inherited = self
                .definitions
                .remove(&definitions::key(query))
                .unwrap_or_else(|| self.visible.last().copied().unwrap_or(false));
            let mut body = inherited;
            if let Some(with) = &query.with {
                if with.recursive {
                    if let Some(cte) = with
                        .cte_tables
                        .iter()
                        .find(|cte| ident(&cte.alias.name) == self.name)
                    {
                        body = definitions::key(&cte.query) == self.target;
                    }
                    for cte in &with.cte_tables {
                        self.definitions.insert(definitions::key(&cte.query), body);
                    }
                } else {
                    for cte in &with.cte_tables {
                        self.definitions.insert(definitions::key(&cte.query), body);
                        if ident(&cte.alias.name) == self.name {
                            body = definitions::key(&cte.query) == self.target;
                        }
                    }
                }
            }
            self.visible.push(body);
            ControlFlow::Continue(())
        }
        fn post_visit_query(&mut self, _: &Query) -> ControlFlow<()> {
            self.visible.pop();
            ControlFlow::Continue(())
        }
        fn pre_visit_table_factor(&mut self, factor: &TableFactor) -> ControlFlow<()> {
            if self.visible.last() == Some(&true)
                && walk::relation(factor)
                    .is_some_and(|relation| relation.cte.as_deref() == Some(&self.name))
            {
                self.factors.insert(factor_key(factor));
            }
            ControlFlow::Continue(())
        }
    }
    let mut references = References {
        name: ident(&target.alias.name),
        target: definitions::key(&target.query),
        visible: Vec::new(),
        definitions: BTreeMap::new(),
        factors: BTreeSet::new(),
    };
    let _ = query.visit(&mut references);
    references.factors
}

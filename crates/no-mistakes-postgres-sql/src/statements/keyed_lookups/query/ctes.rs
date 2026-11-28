use super::{walk_query, ScopeStack};
use crate::fx::FxHashSet;
use crate::idents::ident_key;
use crate::statement_facts::SqlDuplicateKeyedLookupFact;
use sqlparser::ast::{Ident, Query, SetExpr, Visit, Visitor};
use std::ops::ControlFlow;

pub(super) fn walk_reachable(
    query: &Query,
    scopes: &mut ScopeStack,
    out: &mut Vec<SqlDuplicateKeyedLookupFact>,
) {
    let Some(with) = &query.with else {
        return;
    };
    let mut indexed = CandidateUses {
        candidates: with
            .cte_tables
            .iter()
            .map(|cte| ident_key(&cte.alias.name))
            .collect(),
        used: FxHashSet::default(),
    };
    let _ = query.body.visit(&mut indexed);
    let _ = query.order_by.visit(&mut indexed);
    let _ = query.limit_clause.visit(&mut indexed);
    let _ = query.fetch.visit(&mut indexed);
    let mut reachable = Vec::new();
    for cte in with.cte_tables.iter().rev() {
        if with.recursive
            || !matches!(cte.query.body.as_ref(), SetExpr::Select(_))
            || cte.query.with.is_some()
            || indexed.used.contains(&ident_key(&cte.alias.name))
        {
            let _ = cte.query.visit(&mut indexed);
            reachable.push(&cte.query);
        }
    }
    for cte in reachable.into_iter().rev() {
        walk_query(cte, scopes, out);
    }
}

struct CandidateUses {
    candidates: FxHashSet<String>,
    used: FxHashSet<String>,
}

impl Visitor for CandidateUses {
    type Break = ();

    fn pre_visit_ident(&mut self, ident: &Ident) -> ControlFlow<()> {
        let name = ident_key(ident);
        if self.candidates.contains(&name) {
            self.used.insert(name);
        }
        ControlFlow::Continue(())
    }
}

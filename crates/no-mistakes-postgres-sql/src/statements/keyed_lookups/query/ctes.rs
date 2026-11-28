mod recursive;
mod visitor;

use super::{walk_query, ScopeStack};
use crate::fx::FxHashSet;
use crate::idents::ident_key;
use crate::statement_facts::SqlDuplicateKeyedLookupFact;
use sqlparser::ast::{Cte, Expr, Query, SetExpr, Visit, Visitor};

pub(super) fn walk_reachable(
    query: &Query,
    zero_limit: bool,
    scopes: &mut ScopeStack,
    out: &mut Vec<SqlDuplicateKeyedLookupFact>,
) {
    let Some(with) = &query.with else {
        return;
    };
    let mut uses = RelationUses::default();
    if zero_limit {
        let mut current = QueryUses::new(query, None);
        current.body_skipped = true;
        uses.frames.push(current);
        for definition in &with.cte_tables {
            let _ = definition.query.visit(&mut uses);
        }
        let _ = query.limit_clause.visit(&mut uses);
        let _ = query.fetch.visit(&mut uses);
        let _ = uses.post_visit_query(query);
    } else {
        let _ = query.visit(&mut uses);
    }
    for index in uses.root_reachable.into_iter().rev() {
        walk_query(&with.cte_tables[index].query, scopes, out);
    }
}

#[derive(Default)]
struct RelationUses {
    frames: Vec<QueryUses>,
    root_reachable: Vec<usize>,
    blocked_depth: usize,
    blocked_roots: FxHashSet<*const Expr>,
}

struct QueryUses {
    names: FxHashSet<String>,
    definitions: Vec<Definition>,
    parent_definition: Option<usize>,
    recursive: bool,
    body_skipped: bool,
}

struct Definition {
    query: *const Query,
    name: String,
    forced: bool,
    names: FxHashSet<String>,
}

impl QueryUses {
    fn new(query: &Query, parent_definition: Option<usize>) -> Self {
        let definitions = query
            .with
            .iter()
            .flat_map(|with| &with.cte_tables)
            .map(Definition::new)
            .collect();
        Self {
            names: FxHashSet::default(),
            definitions,
            parent_definition,
            body_skipped: false,
            recursive: query.with.as_ref().is_some_and(|with| with.recursive),
        }
    }

    fn resolve(&mut self) -> Vec<usize> {
        if self.body_skipped && self.recursive {
            return self.resolve_recursive_demand();
        }
        let mut reachable = Vec::new();
        for (index, cte) in self.definitions.iter_mut().enumerate().rev() {
            if self.recursive || self.names.remove(&cte.name) || cte.forced {
                self.names.extend(std::mem::take(&mut cte.names));
                reachable.push(index);
            }
        }
        if self.recursive {
            for cte in &self.definitions {
                self.names.remove(&cte.name);
            }
        }
        reachable
    }

    fn named_definition(&self, query: &Query) -> Option<usize> {
        let query = std::ptr::from_ref(query);
        self.definitions.iter().position(|cte| cte.query == query)
    }
}

impl Definition {
    fn new(cte: &Cte) -> Self {
        Self {
            query: std::ptr::from_ref(cte.query.as_ref()),
            name: ident_key(&cte.alias.name),
            forced: modifies_data(&cte.query),
            names: FxHashSet::default(),
        }
    }
}

fn modifies_data(query: &Query) -> bool {
    query
        .with
        .as_ref()
        .is_some_and(|with| with.cte_tables.iter().any(|cte| modifies_data(&cte.query)))
        || match query.body.as_ref() {
            SetExpr::Insert(_) | SetExpr::Update(_) | SetExpr::Delete(_) | SetExpr::Merge(_) => {
                true
            }
            SetExpr::Query(inner) => modifies_data(inner),
            _ => false,
        }
}

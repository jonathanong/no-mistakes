use super::{walk_query, ScopeStack};
use crate::fx::FxHashSet;
use crate::idents::ident_key;
use crate::statement_facts::SqlDuplicateKeyedLookupFact;
use sqlparser::ast::{Cte, Ident, ObjectName, Query, SetExpr, TableFactor, Visit, Visitor};
use std::ops::ControlFlow;

pub(super) fn walk_reachable(
    query: &Query,
    scopes: &mut ScopeStack,
    out: &mut Vec<SqlDuplicateKeyedLookupFact>,
) {
    let Some(with) = &query.with else {
        return;
    };
    let mut uses = RelationUses::default();
    let _ = query.visit(&mut uses);
    for index in uses.root_reachable.into_iter().rev() {
        walk_query(&with.cte_tables[index].query, scopes, out);
    }
}

#[derive(Default)]
struct RelationUses {
    frames: Vec<QueryUses>,
    root_reachable: Vec<usize>,
}

struct QueryUses {
    names: FxHashSet<String>,
    definitions: Vec<Definition>,
    parent_definition: Option<usize>,
    recursive: bool,
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
            recursive: query.with.as_ref().is_some_and(|with| with.recursive),
        }
    }

    fn resolve(&mut self) -> Vec<usize> {
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

fn insert_name(names: &mut FxHashSet<String>, name: Option<&Ident>) {
    if let Some(name) = name {
        names.insert(ident_key(name));
    }
}

fn inspect_table_commands(body: &SetExpr, names: &mut FxHashSet<String>) {
    match body {
        SetExpr::Table(table) if table.schema_name.is_none() => {
            insert_name(names, table.table_name.as_ref());
        }
        SetExpr::SetOperation { left, right, .. } => {
            inspect_table_commands(left, names);
            inspect_table_commands(right, names);
        }
        _ => {}
    }
}

impl Visitor for RelationUses {
    type Break = ();

    fn pre_visit_query(&mut self, query: &Query) -> ControlFlow<()> {
        let parent_definition = self
            .frames
            .last()
            .and_then(|parent| parent.named_definition(query));
        let mut current = QueryUses::new(query, parent_definition);
        inspect_table_commands(&query.body, &mut current.names);
        self.frames.push(current);
        ControlFlow::Continue(())
    }

    fn post_visit_query(&mut self, _: &Query) -> ControlFlow<()> {
        let mut current = self.frames.pop().expect("query visit stack");
        let reachable = current.resolve();
        if let Some(parent) = self.frames.last_mut() {
            match current.parent_definition {
                Some(index) => parent.definitions[index].names = current.names,
                None => parent.names.extend(current.names),
            }
        } else {
            self.root_reachable = reachable;
        }
        ControlFlow::Continue(())
    }

    fn pre_visit_table_factor(&mut self, factor: &TableFactor) -> ControlFlow<()> {
        if let TableFactor::Table {
            name: ObjectName(parts),
            args: None,
            ..
        } = factor
        {
            if let [name] = parts.as_slice() {
                let current = self.frames.last_mut().expect("relation in query visit");
                insert_name(&mut current.names, name.as_ident());
            }
        }
        ControlFlow::Continue(())
    }
}

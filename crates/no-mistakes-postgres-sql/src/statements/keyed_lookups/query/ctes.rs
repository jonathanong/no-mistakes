use super::{walk_query, ScopeStack};
use crate::fx::FxHashSet;
use crate::idents::ident_key;
use crate::statement_facts::SqlDuplicateKeyedLookupFact;
use sqlparser::ast::{Ident, ObjectName, Query, SetExpr, TableFactor, Visit, Visitor};
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
    uses.inspect_table_commands(&query.body);
    let _ = query.body.visit(&mut uses);
    let _ = query.order_by.visit(&mut uses);
    let _ = query.limit_clause.visit(&mut uses);
    let _ = query.fetch.visit(&mut uses);
    let mut reachable = Vec::new();
    for cte in with.cte_tables.iter().rev() {
        if with.recursive
            || modifies_data(&cte.query)
            || uses.names.contains(&ident_key(&cte.alias.name))
        {
            let _ = cte.query.visit(&mut uses);
            reachable.push(&cte.query);
        }
    }
    for cte in reachable.into_iter().rev() {
        walk_query(cte, scopes, out);
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

#[derive(Default)]
struct RelationUses {
    names: FxHashSet<String>,
    scopes: Vec<WithScope>,
}

#[derive(Default)]
struct WithScope {
    declarations: Vec<(String, *const Query)>,
    visible: usize,
    recursive: bool,
}

impl WithScope {
    fn new(query: &Query) -> Self {
        let Some(with) = &query.with else {
            return Self::default();
        };
        let declarations: Vec<_> = with
            .cte_tables
            .iter()
            .map(|cte| {
                (
                    ident_key(&cte.alias.name),
                    std::ptr::from_ref(cte.query.as_ref()),
                )
            })
            .collect();
        Self {
            visible: declarations.len(),
            declarations,
            recursive: with.recursive,
        }
    }

    fn descend(&mut self, query: &Query) {
        if !self.recursive {
            let query = std::ptr::from_ref(query);
            self.visible = self
                .declarations
                .iter()
                .position(|(_, definition)| *definition == query)
                .unwrap_or(self.declarations.len());
        }
    }

    fn shadows(&self, name: &str) -> bool {
        self.declarations[..self.visible]
            .iter()
            .any(|(cte, _)| cte == name)
    }
}

impl RelationUses {
    fn insert(&mut self, name: Option<&Ident>) {
        if let Some(name) = name {
            let name = ident_key(name);
            if !self.scopes.iter().any(|scope| scope.shadows(&name)) {
                self.names.insert(name);
            }
        }
    }

    fn inspect_table_commands(&mut self, body: &SetExpr) {
        match body {
            SetExpr::Table(table) if table.schema_name.is_none() => {
                self.insert(table.table_name.as_ref())
            }
            SetExpr::SetOperation { left, right, .. } => {
                self.inspect_table_commands(left);
                self.inspect_table_commands(right);
            }
            _ => {}
        }
    }
}

impl Visitor for RelationUses {
    type Break = ();

    fn pre_visit_query(&mut self, query: &Query) -> ControlFlow<()> {
        if let Some(parent) = self.scopes.last_mut() {
            parent.descend(query);
        }
        self.scopes.push(WithScope::new(query));
        self.inspect_table_commands(&query.body);
        ControlFlow::Continue(())
    }

    fn post_visit_query(&mut self, _: &Query) -> ControlFlow<()> {
        self.scopes.pop();
        if let Some(parent) = self.scopes.last_mut() {
            parent.visible = parent.declarations.len();
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
                self.insert(name.as_ident());
            }
        }
        ControlFlow::Continue(())
    }
}

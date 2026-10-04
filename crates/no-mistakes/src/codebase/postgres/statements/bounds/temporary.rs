//! Temporary relation identity follows SQL source order, never crossing source boundaries.
mod lifecycle;
mod state;
pub(super) mod view_relations;
use super::items::sql_name;
use crate::codebase::postgres::idents::unwrap_expr;
use crate::codebase::postgres::statements::{
    SqlBoundFact, SqlBoundItemKind, SqlBoundQuery, SqlPinSource,
};
use sqlparser::ast::{Expr, ObjectName, ObjectNamePart, SetExpr, Statement};
use state::{Dependency, State};
use std::collections::BTreeSet;

#[derive(Clone, Default)]
pub(in super::super) struct TemporaryRelations {
    state: State,
    transaction: Option<State>,
    savepoints: Vec<(String, State)>,
}

impl TemporaryRelations {
    pub(in super::super) fn apply(
        &mut self,
        statement: &Statement,
        facts: &mut [SqlBoundFact],
        scope: &super::Scope,
        positions: super::super::value::PlaceholderPositions<'_>,
    ) {
        let mut dependencies = BTreeSet::new();
        if let Statement::CreateView(view) = statement {
            // View declarations have no executed bound fact; collect their source fact here once.
            let declaration = super::query::bound_query(&view.query, scope, positions);
            self.state.dependencies(&declaration, &mut dependencies);
            // The bound projection can omit relations in expressions that do not
            // constrain rows. They still determine a view's lifetime.
            for name in view_relations::names(&view.query) {
                dependencies.insert(if self.state.contains(&name) {
                    Dependency::Temporary(state::key(&name))
                } else {
                    Dependency::Physical(crate::codebase::postgres::decoded_parts(&name))
                });
            }
        }
        self.lifecycle(statement);
        // SELECT INTO's source is resolved before its destination is created.
        for fact in facts {
            self.query(&mut fact.query);
        }
        match statement {
            Statement::CreateTable(table) if table.temporary || temporary_name(&table.name) => {
                let name = sql_name(&table.name);
                let parent = table.partition_of.as_ref().map(sql_name);
                let on_commit_drop = table.on_commit == Some(sqlparser::ast::OnCommit::Drop);
                // Outside an explicit transaction, DROP takes effect at this statement's commit.
                if (!on_commit_drop || self.transaction.is_some())
                    && !(table.if_not_exists
                        && self.state.relations.contains_key(&state::key(&name)))
                    && parent
                        .as_ref()
                        .is_none_or(|parent| self.state.partitioned_parent(parent))
                {
                    self.insert(name.clone());
                    if table.partition_by.is_some() {
                        self.state.partitioned.insert(state::key(&name));
                    }
                    if let Some(parent) = parent {
                        self.state.attach_created_partition(&parent, &name);
                    }
                    if on_commit_drop {
                        self.state.on_commit_drop.insert(state::key(&name));
                    }
                }
            }
            Statement::CreateView(view)
                if view.temporary
                    || dependencies
                        .iter()
                        .any(|dependency| matches!(dependency, Dependency::Temporary(_))) =>
            {
                self.state
                    .relations
                    .insert(state::key(&sql_name(&view.name)), dependencies);
            }
            Statement::Query(query) => {
                if let Some(select) = first_select(&query.body) {
                    if let Some(into) = &select.into {
                        if into.temporary {
                            for target in &into.targets {
                                let parts = match unwrap_expr(target) {
                                    Expr::Identifier(ident) => {
                                        vec![ObjectNamePart::Identifier(ident.clone())]
                                    }
                                    Expr::CompoundIdentifier(idents) => idents
                                        .iter()
                                        .cloned()
                                        .map(ObjectNamePart::Identifier)
                                        .collect(),
                                    _ => continue,
                                };
                                self.insert(sql_name(&ObjectName(parts)));
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn insert(&mut self, name: String) {
        self.state
            .relations
            .insert(state::key(&name), BTreeSet::new());
    }

    fn query(&self, query: &mut SqlBoundQuery) {
        for item in &mut query.items {
            match &mut item.kind {
                SqlBoundItemKind::Table(name) if self.state.contains(name) => {
                    item.kind = SqlBoundItemKind::Opaque;
                    item.pins.clear();
                }
                SqlBoundItemKind::Query(query) => self.query(query),
                _ => {}
            }
            for pin in &mut item.pins {
                if let SqlPinSource::Query(query) = &mut pin.source {
                    self.query(query);
                }
            }
        }
    }
}

// INTO belongs to the first SELECT in a set operation, including parenthesized queries.
fn first_select(expr: &SetExpr) -> Option<&sqlparser::ast::Select> {
    match expr {
        SetExpr::Select(select) => Some(select),
        SetExpr::SetOperation { left, .. } => first_select(left),
        SetExpr::Query(query) => first_select(&query.body),
        _ => None,
    }
}

fn temporary_name(name: &ObjectName) -> bool {
    let parts = crate::codebase::postgres::decoded_parts(&sql_name(name));
    matches!(parts.as_slice(), [schema, _] if schema == "pg_temp")
}

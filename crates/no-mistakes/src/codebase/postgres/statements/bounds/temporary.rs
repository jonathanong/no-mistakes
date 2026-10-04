//! Temporary relation identity follows SQL source order, never crossing source boundaries.
mod identity;
mod lifecycle;
mod state;
pub(super) mod view_relations;
use super::items::sql_name;
use crate::codebase::postgres::idents::unwrap_expr;
use crate::codebase::postgres::statements::{
    SqlBoundFact, SqlBoundItemKind, SqlBoundQuery, SqlPinSource,
};
use crate::codebase::postgres::{SchemaCatalog, SqlViewReads};
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
    pub(in super::super) fn conditional(&self, statement: &Statement) -> bool {
        if self.state.earlier_schemas.is_none() {
            return false;
        }
        match statement {
            Statement::Drop { names, .. } => names
                .iter()
                .any(|name| self.state.possible_temporary(&sql_name(name)).is_some()),
            Statement::AlterTable(table) => self
                .state
                .possible_temporary(&sql_name(&table.name))
                .is_some(),
            _ => false,
        }
    }

    pub(in super::super) fn view_reads(
        statement: &Statement,
        scope: &super::Scope,
        positions: super::super::value::PlaceholderPositions<'_>,
    ) -> Option<SqlViewReads> {
        let Statement::CreateView(view) = statement else {
            return None;
        };
        Some(SqlViewReads {
            query: super::query::bound_query(&view.query, scope, positions),
            names: view_relations::names(&view.query),
        })
    }

    pub(in super::super) fn apply(
        &mut self,
        statement: &Statement,
        facts: &mut [SqlBoundFact],
        view_reads: Option<&SqlViewReads>,
        catalog: Option<&SchemaCatalog>,
    ) {
        let mut dependencies = BTreeSet::new();
        if let Some(reads) = view_reads {
            // View declarations have no executed bound fact; collect their source fact here once.
            self.state.dependencies(&reads.query, &mut dependencies);
            // The bound projection can omit relations in expressions that do not
            // constrain rows. They still determine a view's lifetime.
            for name in &reads.names {
                self.state.include_dependencies(name, &mut dependencies);
            }
        }
        self.lifecycle(statement, catalog);
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
                    let inherited = parent.as_ref().and_then(|parent| {
                        self.state.possible_temporary(parent)?.database_qualifier
                    });
                    self.state.insert(&name, BTreeSet::new(), inherited);
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
            Statement::CreateTable(table) => {
                self.state.restore_physical(&sql_name(&table.name));
            }
            Statement::CreateView(view) => {
                let name = sql_name(&view.name);
                if view.temporary
                    || temporary_name(&view.name)
                    || dependencies.iter().any(|dependency| match dependency {
                        Dependency::Temporary(_) | Dependency::ConditionalTemporary(_, _) => true,
                        Dependency::PossibleTemporary(_) | Dependency::Physical(_) => false,
                    })
                {
                    let inherited = self.state.dependency_database(&dependencies);
                    self.state.insert(&name, dependencies, inherited);
                } else {
                    self.state.restore_physical(&name);
                    self.state.physical_views.insert(
                        crate::codebase::postgres::decoded_parts(&name),
                        dependencies,
                    );
                }
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
        self.state.insert(&name, BTreeSet::new(), None);
    }

    fn query(&self, query: &mut SqlBoundQuery) {
        for item in &mut query.items {
            match &mut item.kind {
                SqlBoundItemKind::Table(name) if self.state.contains(name) => {
                    item.kind = SqlBoundItemKind::Opaque;
                    item.pins.clear();
                }
                SqlBoundItemKind::Table(name) => {
                    item.possible_temporary = self.state.possible_temporary(name);
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
        || identity::database(&sql_name(name)).is_some()
}

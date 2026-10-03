//! Temporary relation identity follows SQL source order, never crossing source boundaries.
mod lifecycle;
mod state;
use super::items::sql_name;
use crate::codebase::postgres::idents::unwrap_expr;
use crate::codebase::postgres::statements::{
    SqlBoundFact, SqlBoundItemKind, SqlBoundQuery, SqlPinSource,
};
use sqlparser::ast::{Expr, ObjectName, ObjectNamePart, SetExpr, Statement};
use state::State;
use std::collections::BTreeSet;

#[derive(Default)]
pub(in super::super) struct TemporaryRelations {
    state: State,
    transaction: Option<State>,
    savepoints: Vec<(String, State)>,
}

impl TemporaryRelations {
    pub(in super::super) fn apply(&mut self, statement: &Statement, facts: &mut [SqlBoundFact]) {
        let mut dependencies = BTreeSet::new();
        for fact in facts.iter() {
            self.state.dependencies(&fact.query, &mut dependencies);
        }
        if let Statement::CreateView(view) = statement {
            // View declarations have no executed bound fact; collect their source fact here once.
            let declaration = super::query::bound_query(&view.query, &super::Scope::default());
            self.state.dependencies(&declaration, &mut dependencies);
        }
        self.lifecycle(statement);
        // SELECT INTO's source is resolved before its destination is created.
        for fact in facts {
            self.query(&mut fact.query);
        }
        match statement {
            Statement::CreateTable(table) if table.temporary => {
                let name = sql_name(&table.name);
                if table.on_commit == Some(sqlparser::ast::OnCommit::Drop) {
                    // Outside an explicit transaction, DROP takes effect at this statement's commit.
                    if self.transaction.is_some() {
                        self.insert(name.clone());
                        self.state.on_commit_drop.insert(state::key(&name));
                    }
                } else {
                    self.insert(name);
                }
            }
            Statement::CreateView(view) if view.temporary || !dependencies.is_empty() => {
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

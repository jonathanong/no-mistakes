//! CTE output names become visible according to PostgreSQL definition order.
use super::super::{
    expressions, locations::Locations, PostgresSqlFunctionReference, PostgresSqlName,
};
use crate::codebase::postgres::idents::ident_key;
use crate::codebase::postgres::statements::TableTokenIndex;
use sqlparser::ast::{Expr, Query, SetExpr, Spanned, Table, TableFactor, Visit, Visitor};
use std::{
    collections::{BTreeMap, BTreeSet},
    ops::ControlFlow,
};

struct Scope {
    bindings: BTreeSet<String>,
    definitions: BTreeMap<usize, String>,
}

pub(super) fn collect(
    query: &Query,
    tokens: &TableTokenIndex,
    locations: &Locations<'_>,
) -> (
    Vec<PostgresSqlName>,
    bool,
    Vec<PostgresSqlFunctionReference>,
) {
    struct Dependencies<'a, 's> {
        locations: &'a Locations<'s>,
        functions: Vec<PostgresSqlFunctionReference>,
        tokens: &'a TableTokenIndex,
        complete: bool,
        scopes: Vec<Scope>,
        relations: BTreeMap<Vec<String>, PostgresSqlName>,
    }
    impl Visitor for Dependencies<'_, '_> {
        type Break = ();
        fn pre_visit_query(&mut self, query: &Query) -> ControlFlow<()> {
            // Inherited bindings stay borrowed through ancestor scopes.
            let mut bindings = BTreeSet::new();
            let definitions: BTreeMap<_, _> = query
                .with
                .iter()
                .flat_map(|with| &with.cte_tables)
                .map(|cte| {
                    (
                        cte.query.as_ref() as *const Query as usize,
                        ident_key(&cte.alias.name),
                    )
                })
                .collect();
            if query.with.as_ref().is_some_and(|with| with.recursive) {
                bindings.extend(definitions.values().cloned());
            }
            self.scopes.push(Scope {
                bindings,
                definitions,
            });
            ControlFlow::Continue(())
        }
        fn post_visit_query(&mut self, query: &Query) -> ControlFlow<()> {
            let mut tables = Vec::new();
            table_arms(&query.body, &mut tables);
            for table in tables {
                if let Some(name) = self.tokens.unique_name(table) {
                    self.relation(expressions::name(&name));
                } else {
                    self.complete = false;
                }
            }
            self.scopes.pop();
            if let Some(parent) = self.scopes.last_mut() {
                if let Some(name) = parent.definitions.get(&(query as *const Query as usize)) {
                    parent.bindings.insert(name.clone());
                }
            }
            ControlFlow::Continue(())
        }
        fn pre_visit_expr(&mut self, expr: &Expr) -> ControlFlow<()> {
            if let Expr::Function(function) = expr {
                self.functions.push(PostgresSqlFunctionReference {
                    name: expressions::name(&function.name),
                    span: self.locations.span(expr.span()),
                });
            }
            ControlFlow::Continue(())
        }
        fn pre_visit_table_factor(&mut self, factor: &TableFactor) -> ControlFlow<()> {
            if let TableFactor::Table {
                name, args: None, ..
            } = factor
            {
                self.relation(expressions::name(name));
            }
            ControlFlow::Continue(())
        }
    }
    impl Dependencies<'_, '_> {
        fn relation(&mut self, fact: PostgresSqlName) {
            let key: Vec<_> = fact
                .parts
                .iter()
                .map(|part| part.identity.clone())
                .collect();
            let cte = key.len() == 1
                && self
                    .scopes
                    .iter()
                    .rev()
                    .any(|scope| scope.bindings.contains(&key[0]));
            if !cte {
                self.relations.entry(key).or_insert(fact);
            }
        }
    }
    let mut dependencies = Dependencies {
        locations,
        functions: Vec::new(),
        tokens,
        complete: true,
        scopes: Vec::new(),
        relations: BTreeMap::new(),
    };
    let _ = query.visit(&mut dependencies);
    dependencies.functions.sort_by_key(|reference| {
        (
            reference
                .name
                .parts
                .iter()
                .map(|part| part.identity.clone())
                .collect::<Vec<_>>(),
            reference.span.as_ref().map(|span| span.start.offset),
        )
    });
    dependencies.functions.dedup();
    (
        dependencies.relations.into_values().collect(),
        dependencies.complete,
        dependencies.functions,
    )
}

fn table_arms<'a>(body: &'a SetExpr, out: &mut Vec<&'a Table>) {
    match body {
        SetExpr::Table(table) => out.push(table),
        SetExpr::SetOperation { left, right, .. } => {
            table_arms(left, out);
            table_arms(right, out);
        }
        // Nested Query wrappers receive their own scoped visitor callbacks.
        _ => (),
    }
}

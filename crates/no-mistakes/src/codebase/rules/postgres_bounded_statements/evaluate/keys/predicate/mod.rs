//! Partial-index implication: literal shapes, parameter forwarding, and enum splits.
use crate::codebase::postgres::idents::unwrap_expr;
use crate::codebase::postgres::statements::SqlBoundItem;
use crate::codebase::postgres::{CatalogTable, SchemaCatalog};
use sqlparser::ast::Expr;
use std::collections::{BTreeMap, BTreeSet};

mod assign;
mod casts;
mod columns;
mod literals;
mod shape;

use columns::column_types;
use shape::{
    atom_with, binding, eq_literal, eq_placeholder, membership_with, plain_column, same, Atom, Lit,
};

pub(super) fn holds(
    predicates: &[Expr],
    item: &SqlBoundItem,
    alternative: usize,
    extra: &[Atom],
    table: &CatalogTable,
    catalog: &SchemaCatalog,
) -> bool {
    let evidence = Evidence::collect(item, alternative, extra, catalog);
    predicates
        .iter()
        .all(|predicate| evidence.implies(predicate, item, alternative, table, catalog))
}

pub(super) fn assignments(
    item: &SqlBoundItem,
    alternative: usize,
    table: &CatalogTable,
    catalog: &SchemaCatalog,
) -> Option<Vec<assign::Assignment>> {
    let evidence = Evidence::collect(item, alternative, &[], catalog);
    assign::expand(
        &evidence.params,
        &evidence.bindings,
        &evidence.conflicts,
        table,
        catalog,
    )
}

struct Evidence {
    atoms: Vec<Atom>,
    params: Vec<(String, String)>,
    bindings: BTreeMap<String, Lit>,
    conflicts: BTreeSet<String>,
    types: BTreeMap<String, String>,
}

impl Evidence {
    fn collect(
        item: &SqlBoundItem,
        alternative: usize,
        extra: &[Atom],
        catalog: &SchemaCatalog,
    ) -> Self {
        let types = column_types(item, catalog);
        let mut evidence = Self {
            atoms: Vec::new(),
            params: Vec::new(),
            bindings: BTreeMap::new(),
            conflicts: BTreeSet::new(),
            types: BTreeMap::new(),
        };
        let branch = item
            .alternative_predicates
            .get(alternative)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        for expr in item.predicate_conjuncts.iter().chain(branch) {
            evidence.classify(expr, catalog, &types);
        }
        evidence.types = types;
        evidence.forward();
        evidence.atoms.extend(extra.iter().cloned());
        evidence
    }

    fn classify(&mut self, expr: &Expr, catalog: &SchemaCatalog, types: &BTreeMap<String, String>) {
        if let Some((column, literal)) = eq_literal(expr, catalog, types) {
            self.atoms.push(Atom::Eq(column, literal));
            return;
        }
        if let Some((column, placeholder)) = eq_placeholder(expr, types) {
            self.params.push((column, placeholder));
            return;
        }
        if let Some((placeholder, literal)) = binding(expr, catalog) {
            self.bind(placeholder, literal);
            return;
        }
        if let Expr::IsNull(column) = unwrap_expr(expr) {
            if let Some(column) = plain_column(column) {
                self.atoms.push(Atom::Null(column));
            }
            return;
        }
        if let Some(atom) = membership_with(expr, catalog, types) {
            self.atoms.push(atom);
        }
    }

    fn forward(&mut self) {
        let forwarded: Vec<_> = self
            .params
            .iter()
            .filter_map(|(column, placeholder)| {
                self.bindings
                    .get(placeholder)
                    .map(|literal| Atom::Eq(column.clone(), literal.clone()))
            })
            .collect();
        self.atoms.extend(forwarded);
    }

    fn bind(&mut self, placeholder: String, literal: Lit) {
        match self.bindings.get(&placeholder) {
            Some(existing) if existing != &literal => {
                self.bindings.remove(&placeholder);
                self.conflicts.insert(placeholder);
            }
            None if !self.conflicts.contains(&placeholder) => {
                self.bindings.insert(placeholder, literal);
            }
            _ => {}
        }
    }

    fn implies(
        &self,
        predicate: &Expr,
        item: &SqlBoundItem,
        alternative: usize,
        table: &CatalogTable,
        catalog: &SchemaCatalog,
    ) -> bool {
        let branch = item
            .alternative_predicates
            .get(alternative)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        if item
            .predicate_conjuncts
            .iter()
            .chain(branch)
            .any(|expr| expr == predicate)
        {
            return true;
        }
        if let Expr::IsNotNull(column) = unwrap_expr(predicate) {
            if let Some(name) = plain_column(column) {
                if super::not_null::proves(&name, item, table, self.strict_not_null(&name)) {
                    return true;
                }
            }
        }
        let Some(need) = atom_with(predicate, catalog, &self.types) else {
            return false;
        };
        self.atoms.iter().any(|have| same(have, &need))
    }

    fn strict_not_null(&self, name: &str) -> bool {
        self.atoms.iter().any(|atom| match atom {
            Atom::Eq(column, _) | Atom::In(column, _) => column == name,
            Atom::Null(_) => false,
        }) || self.params.iter().any(|(column, _)| column == name)
    }
}

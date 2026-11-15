//! Partial-index implication: literal shapes, parameter forwarding, and enum splits.
use crate::codebase::postgres::idents::{ident_key, unwrap_expr};
use crate::codebase::postgres::statements::SqlBoundItem;
use crate::codebase::postgres::{CatalogTable, SchemaCatalog};
use sqlparser::ast::Expr;
use std::collections::{BTreeMap, BTreeSet};

mod assign;
mod shape;

use shape::{atom, binding, eq_literal, eq_placeholder, membership, plain_column, same, Atom, Lit};

pub(super) fn holds(
    predicates: &[Expr],
    item: &SqlBoundItem,
    alternative: usize,
    extra: &[Atom],
) -> bool {
    let evidence = Evidence::collect(item, alternative, extra);
    predicates
        .iter()
        .all(|predicate| evidence.implies(predicate, item, alternative))
}

pub(super) fn assignments(
    item: &SqlBoundItem,
    alternative: usize,
    table: &CatalogTable,
    catalog: &SchemaCatalog,
) -> Option<Vec<assign::Assignment>> {
    let evidence = Evidence::collect(item, alternative, &[]);
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
}

impl Evidence {
    fn collect(item: &SqlBoundItem, alternative: usize, extra: &[Atom]) -> Self {
        let mut evidence = Self {
            atoms: Vec::new(),
            params: Vec::new(),
            bindings: BTreeMap::new(),
            conflicts: BTreeSet::new(),
        };
        let branch = item
            .alternative_predicates
            .get(alternative)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        for expr in item.predicate_conjuncts.iter().chain(branch) {
            evidence.classify(expr);
        }
        evidence.forward();
        evidence.atoms.extend(extra.iter().cloned());
        evidence
    }

    fn classify(&mut self, expr: &Expr) {
        if let Some((column, literal)) = eq_literal(expr) {
            self.atoms.push(Atom::Eq(column, literal));
            return;
        }
        if let Some((column, placeholder)) = eq_placeholder(expr) {
            self.params.push((column, placeholder));
            return;
        }
        if let Some((placeholder, literal)) = binding(expr) {
            self.bind(placeholder, literal);
            return;
        }
        if let Expr::IsNull(column) = unwrap_expr(expr) {
            if let Some(column) = plain_column(column) {
                self.atoms.push(Atom::Null(column));
            }
            return;
        }
        if let Some(atom) = membership(expr) {
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

    fn implies(&self, predicate: &Expr, item: &SqlBoundItem, alternative: usize) -> bool {
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
            if let Expr::Identifier(ident) = unwrap_expr(column) {
                if item.nonnull_columns.contains(&ident_key(ident)) {
                    return true;
                }
            }
        }
        let Some(need) = atom(predicate) else {
            return false;
        };
        self.atoms.iter().any(|have| same(have, &need))
    }
}

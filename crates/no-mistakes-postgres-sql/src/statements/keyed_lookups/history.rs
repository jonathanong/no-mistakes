use super::grouping::grouped_position;
use super::normalize::LookupKey;
use super::scope::ScopeSnapshot;
use crate::idents::{ident_key, unwrap_expr};
use crate::statement_facts::SqlDuplicateKeyedLookupFact;
use sqlparser::ast::{Expr, UnaryOperator};
use std::collections::BTreeMap;
use std::rc::Rc;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Column {
    Bound(usize, String),
    Grouped(usize),
}
type Site = (usize, usize);

const TRUE: u8 = 1;
const FALSE: u8 = 2;
const NULL: u8 = 4;
const MIXED_ROW: u8 = 8;
const ANY: u8 = TRUE | FALSE | NULL | MIXED_ROW;

#[derive(Clone, Default)]
enum Filter {
    #[default]
    None,
    Known(Rc<Expr>),
    Opaque(Rc<()>),
}

impl PartialEq for Filter {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::None, Self::None) => true,
            (Self::Known(left), Self::Known(right)) => left == right,
            (Self::Opaque(left), Self::Opaque(right)) => Rc::ptr_eq(left, right),
            _ => false,
        }
    }
}

impl Eq for Filter {}

impl Filter {
    fn overlaps(&self, other: &Self) -> bool {
        matches!(self, Self::None) || matches!(other, Self::None) || self == other
    }
}

#[derive(Clone, Default, PartialEq, Eq)]
struct Guards {
    columns: BTreeMap<Column, u8>,
    filter: Filter,
}

impl Guards {
    fn matches(&self, other: &Self) -> bool {
        self.filter.overlaps(&other.filter)
            && self.columns.iter().all(|(column, allowed)| {
                allowed & other.columns.get(column).copied().unwrap_or(ANY) != 0
            })
    }
}

#[derive(Clone, Default)]
pub(super) struct LookupHistory {
    active: Guards,
    sites: BTreeMap<LookupKey, Vec<(Guards, Site)>>,
}

impl LookupHistory {
    pub(super) fn new() -> Self {
        Self::default()
    }

    pub(super) fn restrict(&mut self, condition: &Expr, outer: &ScopeSnapshot, taken: bool) {
        let (operand, test, _) = boolean_test(condition);
        let allowed = if taken { test } else { ANY ^ test };
        self.restrict_operand(operand, outer, allowed);
    }

    pub(super) fn restrict_to_null(&mut self, operand: &Expr, outer: &ScopeSnapshot) {
        self.restrict_operand(operand, outer, NULL);
    }

    fn restrict_operand(&mut self, operand: &Expr, outer: &ScopeSnapshot, allowed: u8) {
        if let Some(column) = guard_column(operand, outer) {
            *self.active.columns.entry(column).or_insert(ANY) &= allowed;
        }
    }

    pub(super) fn mark_filtered(&mut self, filter: Option<Expr>) {
        self.active.filter = filter.map_or_else(
            || Filter::Opaque(Rc::new(())),
            |key| Filter::Known(Rc::new(key)),
        );
    }

    pub(super) fn guard_only(&self) -> Self {
        Self {
            active: self.active.clone(),
            sites: BTreeMap::new(),
        }
    }

    pub(super) fn is_reachable(&self) -> bool {
        self.active.columns.values().all(|allowed| *allowed != 0)
    }

    pub(super) fn observe(&mut self, key: &LookupKey, site: Site) -> Option<Site> {
        if !self.is_reachable() {
            return None;
        }
        let sites = self.sites.entry(key.clone()).or_default();
        let first = sites
            .iter()
            .find(|(guards, _)| guards.matches(&self.active))
            .map(|(_, site)| *site);
        if site_is_new(sites, &self.active) {
            sites.push((self.active.clone(), site));
        }
        first
    }

    pub(super) fn merge(&mut self, other: Self) {
        for (key, sites) in other.sites {
            let current = self.sites.entry(key).or_default();
            for (guards, site) in sites {
                if site_is_new(current, &guards) {
                    current.push((guards, site));
                }
            }
        }
    }
}

fn site_is_new(sites: &[(Guards, Site)], guards: &Guards) -> bool {
    sites.iter().all(|(existing, _)| existing != guards)
}

fn boolean_test(expr: &Expr) -> (&Expr, u8, u8) {
    let (operand, when_true) = match unwrap_expr(expr) {
        Expr::UnaryOp {
            op: UnaryOperator::Not,
            expr,
        } => {
            let (operand, when_true, when_false) = boolean_test(expr);
            return (operand, when_false, when_true);
        }
        Expr::IsTrue(expr) => (expr, TRUE),
        Expr::IsFalse(expr) => (expr, FALSE),
        Expr::IsUnknown(expr) | Expr::IsNull(expr) => (expr, NULL),
        Expr::IsNotFalse(expr) => (expr, TRUE | NULL),
        Expr::IsNotTrue(expr) => (expr, FALSE | NULL),
        Expr::IsNotUnknown(expr) | Expr::IsNotNull(expr) => (expr, TRUE | FALSE),
        operand => return (operand, TRUE, FALSE),
    };
    (operand, when_true, ANY ^ when_true)
}

fn guard_column(operand: &Expr, outer: &ScopeSnapshot) -> Option<Column> {
    if let Some(position) = grouped_position(operand) {
        return Some(Column::Grouped(position));
    }
    let Expr::CompoundIdentifier(parts) = unwrap_expr(operand) else {
        return None;
    };
    let [qualifier, column] = parts.as_slice() else {
        return None;
    };
    let column = ident_key(column);
    let binding = outer.resolve_column(&ident_key(qualifier), &column)?;
    Some(Column::Bound(binding, column))
}

pub(super) fn record_lookup(
    key: LookupKey,
    key_display: String,
    line: usize,
    column: usize,
    seen: &mut LookupHistory,
    out: &mut Vec<SqlDuplicateKeyedLookupFact>,
) {
    if let Some((first_line, first_column)) = seen.observe(&key, (line, column)) {
        out.push(SqlDuplicateKeyedLookupFact {
            relation: key.relation,
            key_display,
            first_line,
            first_column,
            line,
            column,
        });
    }
}

//! AST projections preserve quoted identity and literal boundaries without reparsing.
use super::{locations::Locations, types::*};
use sqlparser::ast::{
    Expr, Ident, ObjectName, ObjectNamePart, Spanned, Visit, VisitMut, Visitor, VisitorMut,
};
use std::ops::ControlFlow;

pub(super) fn identifier(ident: &Ident) -> PostgresSqlIdentifier {
    PostgresSqlIdentifier {
        value: ident.value.clone(),
        quoted: ident.quote_style.is_some(),
        identity: if ident.quote_style.is_some() {
            ident.value.clone()
        } else {
            ident.value.to_ascii_lowercase()
        },
    }
}

pub(super) fn name(value: &ObjectName) -> PostgresSqlName {
    PostgresSqlName {
        parts: value
            .0
            .iter()
            .filter_map(ObjectNamePart::as_ident)
            .map(identifier)
            .collect(),
        sql: value.to_string(),
    }
}

pub(super) fn expression(expr: &Expr, locations: &Locations<'_>) -> PostgresSqlExpression {
    let mut refs = References {
        locations,
        columns: Vec::new(),
        functions: Vec::new(),
    };
    let _ = expr.visit(&mut refs);
    refs.columns.sort_by_key(|name| {
        name.parts
            .iter()
            .map(|part| part.identity.clone())
            .collect::<Vec<_>>()
    });
    refs.columns.dedup();
    PostgresSqlExpression {
        sql: expr.to_string(),
        identity: identity(expr),
        span: locations.span(expr.span()),
        columns: refs.columns,
        functions: refs.functions,
    }
}

pub(super) fn identity(expr: &Expr) -> String {
    let mut expr = expr.clone();
    let _ = VisitMut::visit(&mut expr, &mut Canonical);
    let mut structure = serde_json::to_value(&expr).expect("serializable SQL expression");
    remove_locations(&mut structure);
    structure.to_string()
}

fn remove_locations(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(fields) => {
            fields.remove("span");
            for value in fields.values_mut() {
                remove_locations(value);
            }
        }
        serde_json::Value::Array(values) => values.iter_mut().for_each(remove_locations),
        _ => {}
    }
}

struct References<'a, 's> {
    locations: &'a Locations<'s>,
    columns: Vec<PostgresSqlName>,
    functions: Vec<PostgresSqlFunctionReference>,
}
impl Visitor for References<'_, '_> {
    type Break = ();
    fn pre_visit_expr(&mut self, expr: &Expr) -> ControlFlow<()> {
        match expr {
            Expr::Identifier(ident) => self
                .columns
                .push(name(&ObjectName::from(vec![ident.clone()]))),
            Expr::CompoundIdentifier(idents) => {
                self.columns.push(name(&ObjectName::from(idents.clone())))
            }
            Expr::Function(function) => self.functions.push(PostgresSqlFunctionReference {
                name: name(&function.name),
                span: self.locations.span(function.span()),
            }),
            _ => {}
        }
        ControlFlow::Continue(())
    }
}

fn fold(ident: &mut Ident) {
    if ident.quote_style.is_none() {
        ident.value.make_ascii_lowercase();
    }
}
fn fold_name(name: &mut ObjectName) {
    for part in &mut name.0 {
        if let ObjectNamePart::Identifier(ident) = part {
            fold(ident);
        }
    }
}
struct Canonical;
impl VisitorMut for Canonical {
    type Break = ();
    fn post_visit_expr(&mut self, expr: &mut Expr) -> ControlFlow<()> {
        match expr {
            Expr::Identifier(ident) => fold(ident),
            Expr::CompoundIdentifier(idents) => idents.iter_mut().for_each(fold),
            Expr::Function(function) => fold_name(&mut function.name),
            Expr::Nested(inner) => *expr = *inner.clone(),
            _ => {}
        }
        ControlFlow::Continue(())
    }
    fn pre_visit_relation(&mut self, relation: &mut ObjectName) -> ControlFlow<()> {
        fold_name(relation);
        ControlFlow::Continue(())
    }
}

//! Caller-input proof borrows recovered positions and visits every expression once.
use super::conditional_form;
use crate::codebase::postgres::statements::value::{is_placeholder_ident_at, PlaceholderPositions};
use sqlparser::ast::{ArrayElemTypeDef, DataType, Expr, Function, Query, Visit, Visitor};
use std::ops::ControlFlow;

pub(super) fn input_depends_on_data_at(expr: &Expr, positions: PlaceholderPositions<'_>) -> bool {
    struct Found<'a> {
        data: bool,
        positions: PlaceholderPositions<'a>,
        conditional_depth: usize,
    }
    impl Visitor for Found<'_> {
        type Break = ();
        fn pre_visit_query(&mut self, _: &Query) -> ControlFlow<()> {
            self.data = true;
            ControlFlow::Break(())
        }
        fn pre_visit_expr(&mut self, expr: &Expr) -> ControlFlow<()> {
            let data = match expr {
                Expr::Identifier(ident) => !is_placeholder_ident_at(ident, self.positions),
                Expr::CompoundIdentifier(_) => true,
                Expr::Function(function) => {
                    if caller_conditional(function) {
                        self.conditional_depth += 1;
                        false
                    } else {
                        true
                    }
                }
                Expr::Cast { data_type, .. } if self.conditional_depth > 0 => {
                    custom_type(data_type)
                }
                Expr::TypedString(literal) if self.conditional_depth > 0 => {
                    custom_type(&literal.data_type)
                }
                _ => false,
            };
            if data {
                self.data = true;
                return ControlFlow::Break(());
            }
            ControlFlow::Continue(())
        }
        fn post_visit_expr(&mut self, expr: &Expr) -> ControlFlow<()> {
            // Every other call stops traversal in pre_visit_expr.
            if matches!(expr, Expr::Function(_)) {
                self.conditional_depth -= 1;
            }
            ControlFlow::Continue(())
        }
    }
    let mut found = Found {
        data: false,
        positions,
        conditional_depth: 0,
    };
    let _ = expr.visit(&mut found);
    found.data
}

// Conditional syntax cannot be overloaded; window/filter syntax is not that contract.
fn caller_conditional(function: &Function) -> bool {
    conditional_form(function)
        && function.over.is_none()
        && function.filter.is_none()
        && function.within_group.is_empty()
}

// A custom cast/input function can manufacture an array from database state.
// Check only newly accepted conditional inputs, preserving earlier direct-input behavior.
fn custom_type(data_type: &DataType) -> bool {
    match data_type {
        DataType::Custom(_, _) => true,
        DataType::Array(
            ArrayElemTypeDef::AngleBracket(inner)
            | ArrayElemTypeDef::SquareBracket(inner, _)
            | ArrayElemTypeDef::Parenthesis(inner)
            | ArrayElemTypeDef::Qualified(inner, _),
        ) => custom_type(inner),
        _ => false,
    }
}

#[cfg(test)]
mod tests;

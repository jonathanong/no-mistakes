use super::{Engine, State, Value};
use crate::codebase::postgres::embedded::helpers::program::Expr;
mod binary;
mod logical;
impl Engine<'_> {
    pub(super) fn operator(
        &mut self,
        expression: &Expr,
        state: State,
    ) -> Option<Vec<(State, Value)>> {
        match expression {
            Expr::Not(inner) => Some(
                self.expression(inner, state)?
                    .into_iter()
                    .map(|(state, value)| {
                        (
                            state,
                            value
                                .truth()
                                .map_or(Value::Scalar, |value| Value::Bool(!value)),
                        )
                    })
                    .collect(),
            ),
            Expr::And(a, b) | Expr::Or(a, b) | Expr::Nullish(a, b) => {
                self.logical(expression, a, b, state)
            }
            Expr::Add(a, b) | Expr::Eq(a, b, _, _) => self.binary(expression, a, b, state),
            _ => None,
        }
    }
}
#[cfg(test)]
mod tests;

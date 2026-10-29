//! Strict equality narrows one immutable semantic slot, without trusting TS types.
use super::{Engine, State, Value};
use crate::codebase::postgres::embedded::helpers::program::Expr;

pub(super) enum Refinement {
    Applied(Vec<(State, Value)>),
    Unchanged(State),
}

impl Engine<'_> {
    pub(super) fn refine_strict(
        &mut self,
        expression: &Expr,
        a: &Expr,
        b: &Expr,
        left: &Value,
        right: &Value,
        mut state: State,
    ) -> Refinement {
        let Expr::Eq(_, _, true, negate) = expression else {
            return Refinement::Unchanged(state);
        };
        let (name, text) = match (a, b, left, right) {
            (Expr::Ident(name), _, Value::Scalar, Value::Text(text))
            | (_, Expr::Ident(name), Value::Text(text), Value::Scalar) => (name, text),
            _ => return Refinement::Unchanged(state),
        };
        if state
            .excluded_text
            .get(name)
            .is_some_and(|values| values.contains(&text.raw.sql))
        {
            return Refinement::Applied(vec![(state, Value::Bool(*negate))]);
        }
        let Some(mut equal) = self.copy_state(&state) else {
            self.failures += 1;
            return Refinement::Applied(Vec::new());
        };
        if self.sql_bytes(text.raw.sql.len()).is_none() {
            self.failures += 1;
            return Refinement::Applied(Vec::new());
        }
        equal.env.insert(name.clone(), Value::Text(text.clone()));
        equal.excluded_text.remove(name);
        state
            .excluded_text
            .entry(name.clone())
            .or_default()
            .push(text.raw.sql.clone());
        Refinement::Applied(vec![
            (equal, Value::Bool(!*negate)),
            (state, Value::Bool(*negate)),
        ])
    }
}

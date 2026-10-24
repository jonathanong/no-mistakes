use super::{Engine, State, Value};
use crate::codebase::postgres::embedded::helpers::program::Expr;

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
                let mut out = Vec::new();
                for (state, value) in self.expression(a, state)? {
                    let take_right = match expression {
                        Expr::And(..) => value.truth(),
                        Expr::Or(..) => value.truth().map(|truth| !truth),
                        _ => match value {
                            Value::Null | Value::Undefined => Some(true),
                            Value::Scalar
                            | Value::OpaqueMember
                            | Value::Constrained { .. }
                            | Value::MaybeExecutor => None,
                            _ => Some(false),
                        },
                    };
                    match take_right {
                        Some(true) => out.extend(self.expression(b, state)?),
                        Some(false) => out.push((state, value)),
                        None => {
                            let retained = match expression {
                                Expr::And(..) => Value::Constrained {
                                    truth: false,
                                    executor_payload: false,
                                },
                                Expr::Or(..) => Value::Constrained {
                                    truth: true,
                                    executor_payload: state.contains_executor(&value),
                                },
                                _ if matches!(value, Value::MaybeExecutor) => Value::Executor,
                                _ => value,
                            };
                            if let Some(copied) = self.copy_state(&state) {
                                out.push((copied, retained));
                            } else {
                                self.failures += 1;
                            }
                            out.extend(self.expression(b, state)?);
                        }
                    }
                }
                Some(out)
            }
            Expr::Add(a, b) | Expr::Eq(a, b, _, _) => {
                let mut out = Vec::new();
                for (state, values) in self.all(&[a, b], state)? {
                    let [left, right] = values.as_slice() else {
                        return None;
                    };
                    let value = match expression {
                        Expr::Add(..) => {
                            let (Value::Text(left), Value::Text(right)) = (left, right) else {
                                self.failures += 1;
                                continue;
                            };
                            let Some(bytes) = left.raw.sql.len().checked_add(right.raw.sql.len())
                            else {
                                self.failures += 1;
                                continue;
                            };
                            if self.sql_bytes(bytes).is_none() {
                                self.failures += 1;
                                continue;
                            }
                            let mut text = left.clone();
                            if text.append(right.clone()).is_none() {
                                self.failures += 1;
                                continue;
                            }
                            Value::Text(text)
                        }
                        Expr::Eq(_, _, strict, negate) => equals(left, right, *strict)
                            .map_or(Value::Scalar, |value| Value::Bool(value != *negate)),
                        _ => return None,
                    };
                    out.push((state, value));
                }
                Some(out)
            }
            _ => None,
        }
    }
}
fn equals(a: &Value, b: &Value, strict: bool) -> Option<bool> {
    let nullish = |value: &Value| matches!(value, Value::Null | Value::Undefined);
    match (a, b) {
        (
            Value::Scalar | Value::OpaqueMember | Value::Constrained { .. } | Value::MaybeExecutor,
            _,
        )
        | (
            _,
            Value::Scalar | Value::OpaqueMember | Value::Constrained { .. } | Value::MaybeExecutor,
        ) => None,
        (Value::Text(a), Value::Text(b)) => Some(a.raw.sql == b.raw.sql),
        (Value::Bool(a), Value::Bool(b)) => Some(a == b),
        (Value::Null, Value::Null) | (Value::Undefined, Value::Undefined) => Some(true),
        _ if !strict && nullish(a) && nullish(b) => Some(true),
        (
            Value::Object(_)
            | Value::Array(_)
            | Value::Builder(_)
            | Value::Sql(_)
            | Value::Executor
            | Value::Promise(_),
            _,
        )
        | (
            _,
            Value::Object(_)
            | Value::Array(_)
            | Value::Builder(_)
            | Value::Sql(_)
            | Value::Executor
            | Value::Promise(_),
        ) => None,
        _ if strict => Some(false),
        _ => None,
    }
}

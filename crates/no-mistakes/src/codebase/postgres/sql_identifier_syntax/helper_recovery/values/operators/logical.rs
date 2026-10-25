use super::*;
impl Engine<'_> {
    pub(super) fn logical(
        &mut self,
        expression: &Expr,
        a: &Expr,
        b: &Expr,
        state: State,
    ) -> Option<Vec<(State, Value)>> {
        let mut out = Vec::new();
        for (state, value) in self.expression(a, state)? {
            let take_right = take_right(expression, &value);
            match take_right {
                Some(true) => out.extend(self.expression(b, state)?),
                Some(false) => out.push((state, value)),
                None => {
                    let retained = retained(expression, &state, value);
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
}

fn take_right(expression: &Expr, value: &Value) -> Option<bool> {
    match expression {
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
    }
}

fn retained(expression: &Expr, state: &State, value: Value) -> Value {
    match expression {
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
    }
}

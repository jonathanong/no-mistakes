use super::*;
impl Engine<'_> {
    pub(super) fn binary(
        &mut self,
        expression: &Expr,
        a: &Expr,
        b: &Expr,
        state: State,
    ) -> Option<Vec<(State, Value)>> {
        let mut out = Vec::new();
        for (state, values) in self.all(&[a, b], state)? {
            let [left, right] = values.as_slice() else {
                return None;
            };
            let value = match expression {
                Expr::Add(..) => match self.concatenate(left, right) {
                    Some(value) => value,
                    None => {
                        self.failures += 1;
                        continue;
                    }
                },
                Expr::Eq(_, _, strict, negate) => equals(left, right, *strict)
                    .map_or(Value::Scalar, |value| Value::Bool(value != *negate)),
                _ => return None,
            };
            out.push((state, value));
        }
        Some(out)
    }
    fn concatenate(&mut self, left: &Value, right: &Value) -> Option<Value> {
        let (Value::Text(left), Value::Text(right)) = (left, right) else {
            return None;
        };
        let bytes = left.raw.sql.len().checked_add(right.raw.sql.len())?;
        self.sql_bytes(bytes)?;
        let mut text = left.clone();
        text.append(right.clone())?;
        Some(Value::Text(text))
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

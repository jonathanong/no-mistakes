use super::{Engine, State, Value};
use crate::codebase::postgres::embedded::helpers::program::Expr;
impl Engine<'_> {
    pub(super) fn member(
        &mut self,
        object: &Expr,
        name: &str,
        state: State,
    ) -> Option<Vec<(State, Value)>> {
        let mut out = Vec::new();
        for (state, value) in self.expression(object, state)? {
            if let Some(value) = value.member(name) {
                out.push((state, value));
            } else {
                self.failures += 1;
            }
        }
        Some(out)
    }
    pub(super) fn append(
        &mut self,
        target: &Expr,
        argument: &Expr,
        state: State,
    ) -> Option<Vec<(State, Value)>> {
        let mut out = Vec::new();
        for (mut state, values) in self.all(&[target, argument], state)? {
            let [Value::Builder(id), value] = values.as_slice() else {
                self.failures += 1;
                continue;
            };
            let Some(suffix) = state.text(value) else {
                self.failures += 1;
                continue;
            };
            let Some(target) = state.heap.get(*id) else {
                self.failures += 1;
                continue;
            };
            let Some(bytes) = target.raw.sql.len().checked_add(suffix.raw.sql.len()) else {
                self.failures += 1;
                continue;
            };
            if self.sql_bytes(bytes).is_none() || state.heap.get_mut(*id)?.append(suffix).is_none()
            {
                self.failures += 1;
                continue;
            }
            out.push((state, Value::Builder(*id)));
        }
        Some(out)
    }
    pub(super) fn conditional(
        &mut self,
        test: &Expr,
        yes: &Expr,
        no: &Expr,
        state: State,
    ) -> Option<Vec<(State, Value)>> {
        let mut out = Vec::new();
        for (state, value) in self.expression(test, state)? {
            match value.truth() {
                Some(true) => out.extend(self.expression(yes, state)?),
                Some(false) => out.extend(self.expression(no, state)?),
                None => {
                    if let Some(copied) = self.copy_state(&state) {
                        out.extend(self.expression(yes, copied)?);
                    } else {
                        self.failures += 1;
                    }
                    out.extend(self.expression(no, state)?);
                }
            }
        }
        Some(out)
    }
}

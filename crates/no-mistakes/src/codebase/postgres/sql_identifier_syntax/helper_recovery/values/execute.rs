use super::*;
impl Engine<'_> {
    pub(super) fn cap<T>(&mut self, values: &mut Vec<T>) {
        if self.budget.variants(values.len()).is_err() {
            self.failures += 1;
            values.truncate(self.budget.max_variants());
        }
    }
    pub(super) fn copy_state(&mut self, state: &State) -> Option<State> {
        self.sql_bytes(state.bytes())?;
        Some(state.clone())
    }
    pub(super) fn program(&mut self, lowered: &Lowered, args: &[Value]) -> Option<Vec<Value>> {
        let mut states = vec![State::default()];
        for (index, param) in lowered.program.params.iter().enumerate() {
            let mut next = Vec::new();
            for state in states {
                let values = match args.get(index) {
                    Some(Value::Undefined) | None => match &param.default {
                        Some(default) => self.expression(default, state)?,
                        None => vec![(state, Value::Undefined)],
                    },
                    Some(value) => vec![(state, value.clone())],
                };
                for (mut state, value) in values {
                    state.env.insert(param.name.clone(), value);
                    next.push(state);
                }
            }
            self.cap(&mut next);
            states = next;
        }
        let mut values = Vec::new();
        for state in self.block(&lowered.program.body, states)? {
            let Some(value) = state.materialize(state.returned.clone().unwrap_or(Value::Undefined))
            else {
                self.failures += 1;
                continue;
            };
            if !values.contains(&value) {
                values.push(value);
            }
        }
        Some(values)
    }
    pub(super) fn sql_bytes(&mut self, bytes: usize) -> Option<()> {
        self.budget
            .charge(
                Cost {
                    work: 0,
                    depth: 0,
                    sql_bytes: bytes,
                },
                self.stack.len(),
            )
            .ok()
    }
    pub(super) fn charge(&mut self) -> Option<()> {
        self.budget
            .charge(
                Cost {
                    work: 1,
                    depth: 0,
                    sql_bytes: 0,
                },
                self.stack.len(),
            )
            .ok()
    }
}

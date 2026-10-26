use super::{Engine, State, Value};
use crate::codebase::postgres::embedded::helpers::program::Stmt;

impl Engine<'_> {
    pub(super) fn block(&mut self, body: &[Stmt], mut states: Vec<State>) -> Option<Vec<State>> {
        for statement in body {
            let mut next = Vec::new();
            for state in states {
                if state.returned.is_some() {
                    next.push(state);
                    continue;
                }
                if self.charge().is_none() {
                    self.failures += 1;
                    continue;
                }
                match statement {
                    Stmt::Let(name, value) => {
                        for (mut state, value) in self.expression(value, state)? {
                            state.env.insert(name.clone(), value);
                            next.push(state);
                        }
                    }
                    Stmt::Expr(value) => next.extend(
                        self.expression(value, state)?
                            .into_iter()
                            .map(|(state, _)| state),
                    ),
                    Stmt::Return(value, _) => {
                        let values = match value {
                            Some(value) => self.expression(value, state)?,
                            None => vec![(state, Value::Undefined)],
                        };
                        for (mut state, value) in values {
                            state.returned = Some(value);
                            next.push(state);
                        }
                    }
                    Stmt::If(test, yes, no) => {
                        for (state, value) in self.expression(test, state)? {
                            match value.truth() {
                                Some(true) => next.extend(self.block(yes, vec![state])?),
                                Some(false) => next.extend(self.block(no, vec![state])?),
                                None => {
                                    if let Some(copied) = self.copy_state(&state) {
                                        next.extend(self.block(yes, vec![copied])?);
                                    } else {
                                        self.failures += 1;
                                    }
                                    next.extend(self.block(no, vec![state])?);
                                }
                            }
                        }
                    }
                    // Unsupported effects never inherit a value-only exemption.
                    _ => {
                        self.failures += 1;
                        continue;
                    }
                }
                self.cap(&mut next);
            }
            states = next;
        }
        Some(states)
    }
}

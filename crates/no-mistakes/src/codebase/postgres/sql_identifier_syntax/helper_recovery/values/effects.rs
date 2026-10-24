//! Executable transport proof uses the same closed evaluator, with explicit tokens.
use super::super::{CallModel, Recovered};
use super::*;
use crate::codebase::postgres::embedded::helpers::program::{Expr, Stmt};
use crate::fx::FxHashSet;

impl Engine<'_> {
    pub fn transport(
        &mut self,
        file: &Path,
        call: &CallModel,
        executors: &FxHashSet<usize>,
    ) -> Option<Recovered> {
        if !call.members.is_empty() {
            return None;
        }
        let key = self.catalog.callee(file, call.callee?)?;
        self.effects = true;
        self.root_file = Some(file.to_path_buf());
        let mut state = State::default();
        for argument in &call.arguments {
            for (id, _) in &argument.references {
                state.env.insert(
                    format!("binding:{id}"),
                    if executors.contains(id) {
                        Value::Executor
                    } else {
                        Value::Scalar
                    },
                );
            }
        }
        let expressions = call
            .arguments
            .iter()
            .map(|arg| {
                let [Stmt::Return(Some(value), _)] = arg.program.body.as_slice() else {
                    return None;
                };
                Some(value)
            })
            .collect::<Option<Vec<_>>>()?;
        let states = match self.all(&expressions, state) {
            Some(states) => states,
            None => {
                self.failures += 1;
                Vec::new()
            }
        };
        for (state, args) in states {
            let Some(args) = args
                .into_iter()
                .map(|value| state.materialize(value))
                .collect::<Option<Vec<_>>>()
            else {
                self.failures += 1;
                continue;
            };
            let Some(values) = self.run(&key, &args) else {
                self.failures += 1;
                continue;
            };
            if values.iter().any(Value::contains_executor) {
                self.failures += 1;
            }
        }
        Some(Recovered {
            alternatives: self
                .executed
                .iter()
                .map(|event| event.sql.raw.clone())
                .collect(),
            residual: self.residual() || self.executed.iter().any(|event| !event.sql.valid_binds()),
            nonexecuting: self.executed.is_empty() && !self.residual(),
        })
    }
    pub(super) fn call(
        &mut self,
        name: &str,
        args: &[Expr],
        state: State,
    ) -> Option<Vec<(State, Value)>> {
        if name == "operation:await" {
            let [argument] = args else {
                return None;
            };
            let mut out = Vec::new();
            for (state, value) in self.expression(argument, state)? {
                let value = match value {
                    Value::Promise(value) => *value,
                    Value::Text(_) | Value::Bool(_) | Value::Null | Value::Undefined => value,
                    _ => {
                        self.failures += 1;
                        continue;
                    }
                };
                out.push((state, value));
            }
            return Some(out);
        }
        let (name, site) = name.rsplit_once('@')?;
        let site = site.parse().ok()?;
        let (slot, member) = name
            .split_once('.')
            .map_or((name, None), |(root, member)| (root, Some(member)));
        let id = slot.strip_prefix("binding:")?.parse().ok()?;
        let sink = match state.env.get(slot) {
            Some(Value::Executor | Value::MaybeExecutor) => true,
            Some(_) => false,
            None => self.catalog.sink(self.file()?, id),
        };
        if sink && self.effects && (member.is_none() || member == Some("query")) {
            let mut out = Vec::new();
            for (state, args) in self.all(&args.iter().collect::<Vec<_>>(), state)? {
                match args.first().and_then(|arg| state.text(arg)) {
                    Some(text) => self.executed.push(Execution {
                        file: self.file()?.to_path_buf(),
                        start: site,
                        sql: text,
                    }),
                    None => self.failures += 1,
                }
                out.push((state, Value::Promise(Box::new(Value::Scalar))));
            }
            return Some(out);
        }
        if sink || member.is_some() {
            return None;
        }
        let key = self.catalog.callee(self.file()?, id)?;
        let mut out = Vec::new();
        for (state, args) in self.all(&args.iter().collect::<Vec<_>>(), state)? {
            let Some(values) = self.run(&key, &args) else {
                self.failures += 1;
                continue;
            };
            for value in values {
                let Some(mut state) = self.copy_state(&state) else {
                    self.failures += 1;
                    continue;
                };
                let value = match value {
                    Value::Sql(text) => state.allocate(text),
                    other => other,
                };
                out.push((state, value));
            }
        }
        Some(out)
    }
}

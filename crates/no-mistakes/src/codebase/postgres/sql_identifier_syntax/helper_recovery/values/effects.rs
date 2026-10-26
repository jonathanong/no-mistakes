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
        let (alternatives, execution_sites) = self
            .executed
            .iter()
            .map(|event| (event.sql.raw.clone(), Some(event.site())))
            .unzip();
        Some(Recovered {
            alternatives,
            execution_sites,
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
            return self.await_result(args, state);
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
            return self.execute_sql(args, state, site);
        }
        if sink || member.is_some() {
            return None;
        }
        self.invoke_helper(id, args, state)
    }
}

include!("effects/calls.rs");

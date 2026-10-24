//! Closed low-only evaluation: every call must resolve, including statements.
mod effects;
mod execute;
mod expressions;
mod forms;
mod operators;
mod sites;
mod statements;
mod templates;
use super::{
    budget::{Budget, Cost, Limits},
    lower::Lowered,
    state::State,
    value::Value,
};
use crate::codebase::dependencies::extract::CallableId;
use crate::fx::FxHashMap;
use rustc_hash::FxHasher;
use std::{
    hash::Hasher,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct Key {
    pub file: PathBuf,
    pub callable: CallableId,
}
/// A resolver/catalog view, not a second relationship graph.
pub(super) trait Catalog {
    fn function(&self, key: &Key) -> Option<&Lowered>;
    fn callee(&self, from: &Path, binding: usize) -> Option<Key>;
    fn sink(&self, _from: &Path, _binding: usize) -> bool {
        false
    }
}
#[derive(Clone)]
struct Execution {
    file: PathBuf,
    start: u32,
    sql: super::sql_text::SqlText,
}
#[derive(Clone)]
struct Summary {
    args: Vec<Value>,
    outcomes: Vec<Value>,
    cost: Cost,
    executed: Vec<Execution>,
}
#[derive(Default)]
pub(super) struct Cache {
    complete: FxHashMap<(Key, u64), Vec<Summary>>,
}
pub(super) struct Engine<'a> {
    catalog: &'a dyn Catalog,
    cache: &'a mut Cache,
    budget: Budget,
    stack: Vec<Key>,
    deepest: usize,
    root_file: Option<PathBuf>,
    failures: usize,
    effects: bool,
    executed: Vec<Execution>,
}
impl<'a> Engine<'a> {
    pub fn new(catalog: &'a dyn Catalog, cache: &'a mut Cache, limits: Limits) -> Self {
        Self {
            catalog,
            cache,
            budget: Budget::new(limits),
            stack: Vec::new(),
            deepest: 0,
            root_file: None,
            failures: 0,
            effects: false,
            executed: Vec::new(),
        }
    }
    pub fn evaluate(&mut self, file: &Path, lowered: &Lowered) -> Option<Vec<Value>> {
        self.root_file = Some(file.to_path_buf());
        let mut state = State::default();
        for (id, _) in &lowered.references {
            state.env.insert(format!("binding:{id}"), Value::Scalar);
        }
        let mut values = Vec::new();
        for state in self.block(&lowered.program.body, vec![state])? {
            let Some(value) = state.materialize(state.returned.clone().unwrap_or(Value::Undefined))
            else {
                self.failures += 1;
                continue;
            };
            if !values.contains(&value) {
                values.push(value);
            }
        }
        (!values.is_empty()).then_some(values)
    }
    pub fn residual(&self) -> bool {
        self.failures > 0
    }
    fn file(&self) -> Option<&Path> {
        self.stack
            .last()
            .map(|key| key.file.as_path())
            .or(self.root_file.as_deref())
    }
    pub fn run(&mut self, key: &Key, args: &[Value]) -> Option<Vec<Value>> {
        if self.stack.contains(key) || !args.iter().all(Value::read_only_argument) {
            return None;
        }
        let failures = self.failures;
        let executed = self.executed.len();
        let before = self.budget.used();
        let before_bytes = self.budget.bytes();
        let mut hash = FxHasher::default();
        for arg in args {
            arg.cache_hash(&mut hash);
        }
        let cache_key = (key.clone(), hash.finish());
        if let Some(summary) = self
            .cache
            .complete
            .get(&cache_key)
            .and_then(|bucket| bucket.iter().find(|entry| entry.args == args))
        {
            self.budget.charge(summary.cost, self.stack.len()).ok()?;
            self.deepest = self.deepest.max(self.stack.len() + summary.cost.depth);
            self.executed.extend(summary.executed.clone());
            return Some(summary.outcomes.clone());
        }
        self.sql_bytes(args.iter().map(Value::bytes).sum())?;
        let lowered = self.catalog.function(key)?.clone();
        self.budget
            .charge(
                Cost {
                    work: 1,
                    depth: 1,
                    sql_bytes: 0,
                },
                self.stack.len(),
            )
            .ok()?;
        self.stack.push(key.clone());
        let ancestor_deepest = self.deepest;
        self.deepest = self.stack.len();
        let result = self.program(&lowered, args);
        let depth = self.deepest - self.stack.len() + 1;
        self.stack.pop();
        self.deepest = self.deepest.max(ancestor_deepest);
        let mut outcomes = result?;
        if lowered.asynchronous {
            for value in &mut outcomes {
                if !matches!(value, Value::Promise(_)) {
                    *value = Value::Promise(Box::new(value.clone()));
                }
            }
        }
        self.cap(&mut outcomes);
        let cost = Cost {
            work: self.budget.used() - before,
            depth,
            sql_bytes: self.budget.bytes() - before_bytes,
        };
        if failures == self.failures {
            self.cache
                .complete
                .entry(cache_key)
                .or_default()
                .push(Summary {
                    args: args.to_vec(),
                    outcomes: outcomes.clone(),
                    cost,
                    executed: self.executed[executed..].to_vec(),
                });
        }
        Some(outcomes)
    }
}

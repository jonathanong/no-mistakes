use super::super::{Environment, Evaluator, Function, Value};
use super::state::Snapshot;
use crate::codebase::postgres::query_annotation::evaluate::Scope;
use crate::codebase::postgres::query_annotation::{Expr, Step};
use crate::fx::FxHashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

type Key = (PathBuf, Vec<(Environment, String, bool)>);
type RootValues = Vec<(Option<Value>, bool)>;
#[derive(Default)]
pub(super) struct Memo {
    entries: FxHashMap<Key, (RootValues, Arc<Snapshot>)>,
    pub constructions: usize,
}
impl Memo {
    fn cached(&self, key: &Key, roots: &RootValues) -> Option<Arc<Snapshot>> {
        let (before, snapshot) = self.entries.get(key)?;
        (before == roots).then(|| snapshot.clone())
    }
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

struct Inputs<'a> {
    captured: &'a FxHashMap<Environment, FxHashMap<String, Environment>>,
    scopes: &'a [Scope],
    fresh: &'a FxHashMap<Environment, crate::fx::FxHashSet<String>>,
}
impl Inputs<'_> {
    fn roots(&self, path: &Path, captured: Environment, function: &Function) -> (Key, RootValues) {
        let reads = super::dependencies::names(function);
        let mut names = reads
            .values
            .into_iter()
            .map(|name| (name, true))
            .chain(reads.identities.into_iter().map(|name| (name, false)))
            .map(|(name, read)| {
                let owner = self
                    .captured
                    .get(&captured)
                    .and_then(|origins| origins.get(&name))
                    .copied()
                    .unwrap_or(captured);
                (owner, name, read)
            })
            .collect::<Vec<_>>();
        names.sort_unstable();
        let roots = names
            .iter()
            .map(|(env, name, _)| {
                (
                    self.scopes[*env].get(name).cloned(),
                    self.fresh
                        .get(env)
                        .is_some_and(|names| names.contains(name)),
                )
            })
            .collect::<Vec<_>>();
        let key = (path.to_path_buf(), names);
        (key, roots)
    }
}
impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn memoized_callback_snapshot(
        &mut self,
        path: &Path,
        captured: Environment,
        function: &Function,
        depth: u8,
        memo: &mut Memo,
    ) -> Arc<Snapshot> {
        let (key, roots) = Inputs {
            captured: &self.captured_bindings,
            scopes: &self.scopes,
            fresh: &self.fresh_mapped_parameters,
        }
        .roots(path, captured, function);
        if let Some(snapshot) = memo.cached(&key, &roots) {
            return snapshot;
        }
        let snapshot = Arc::new(self.imported_callback_snapshot(path, captured, function, depth));
        memo.constructions += 1;
        memo.entries.insert(key, (roots, snapshot.clone()));
        snapshot
    }
    pub(super) fn callback_may_mutate(
        &self,
        function: &Function,
        path: &Path,
        env: Environment,
    ) -> bool {
        Classification {
            executors: &self.files[path].executors,
            effect: &|expr| self.effect_can_mutate(expr, path, env),
        }
        .summary(function)
    }
}

struct Classification<'a> {
    executors: &'a crate::fx::FxHashSet<u32>,
    effect: &'a dyn Fn(&Expr) -> bool,
}
impl Classification<'_> {
    fn summary(&self, function: &Function) -> bool {
        function.body.iter().any(|step| match step {
            Step::Bind(_, expr)
            | Step::Hoisted(_, expr)
            | Step::Return(expr)
            | Step::Effect(expr) => self.expression(expr),
            Step::Reserve(_) | Step::Var(_) => false,
            _ => true,
        })
    }
    fn expression(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Call { start, args, .. } if self.executors.contains(start) => {
                args.iter().any(|expr| self.expression(expr))
            }
            Expr::Children(parts) | Expr::Template(parts) | Expr::Sequence(parts) => {
                parts.iter().any(|expr| self.expression(expr))
            }
            _ => (self.effect)(expr),
        }
    }
}

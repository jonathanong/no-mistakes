//! Closed local factories may fork; the original source-order fold still joins them.
mod dispatch;
mod fork;
mod gate;
mod outcome;
#[cfg(test)]
mod probe;
#[cfg(not(test))]
#[path = "probe/runtime.rs"]
mod probe;
#[cfg(test)]
mod tests;
mod translate;
use super::super::{Environment, Evaluator, Value};
use crate::codebase::postgres::query_annotation::Expr;
pub(in crate::codebase::postgres::query_annotation::evaluate) use outcome::Outcome;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
pub(in crate::codebase::postgres::query_annotation::evaluate) type Handler<'a, F> =
    fn(&mut Evaluator<'a, F>, &[Expr], &Path, &Environment, u8, bool) -> Option<Value>;
impl<'a, F: Fn(&str, &Path) -> Option<PathBuf> + Sync> Evaluator<'a, F> {
    pub(in crate::codebase::postgres::query_annotation) fn enable_parallel_arms(&mut self) {
        self.parallel_arms = Some(run::<F>);
    }
}
fn run<F: Fn(&str, &Path) -> Option<PathBuf> + Sync>(
    evaluator: &mut Evaluator<'_, F>,
    arms: &[Expr],
    path: &Path,
    env: &Environment,
    depth: u8,
    generic: bool,
) -> Option<Value> {
    if rayon::current_num_threads() == 1 {
        return None;
    }
    if !gate::eligible(evaluator, arms, path, *env, depth) {
        return None;
    }
    let probe = probe::current();
    let observer = crate::diagnostics::current();
    if let Some(observer) = &observer {
        observer.increment(
            "postgres.annotation.parallel_factory_arms",
            arms.len() as u64,
        );
    }
    let prepared = arms
        .par_iter()
        .with_max_len(1)
        .map(|arm| {
            probe.enter();
            crate::diagnostics::with_observer(observer.clone(), || {
                let mut fork = evaluator.fork_arm();
                let value = fork.expr(arm, path, env, depth, generic);
                Outcome::take(fork, value, evaluator.scopes.len(), evaluator.next_builder)
            })
        })
        .collect::<Vec<_>>();
    Some(evaluator.alternatives_prepared(arms, path, env, depth, generic, Some(prepared)))
}

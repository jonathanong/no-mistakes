use super::super::{scope::StateWeak, Environment, Scope};
use super::CapturedOrigins;
use crate::fx::FxHashSet;

#[derive(Debug)]
pub(super) struct State {
    frame: StateWeak,
    origins: Vec<(Environment, StateWeak)>,
}

impl CapturedOrigins {
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn refresh_is_current(
        &self,
        frame: &Scope,
        scopes: &[Scope],
    ) -> bool {
        let Ok(memo) = self.0.refresh.lock() else {
            return false;
        };
        memo.as_ref().is_some_and(|state| {
            state.frame.matches(frame)
                && state.origins.iter().all(|(id, previous)| {
                    scopes.get(*id).is_some_and(|scope| previous.matches(scope))
                })
        })
    }

    pub(in crate::codebase::postgres::query_annotation::evaluate) fn remember_refresh(
        &self,
        frame: &Scope,
        origins: &FxHashSet<Environment>,
        scopes: &[Scope],
    ) {
        let Ok(mut memo) = self.0.refresh.lock() else {
            return;
        };
        // The exact refresh loop owns the dependency set. Invalid indices must
        // decline the proof, rather than omit a dependency or retain an old hit.
        let dependencies = origins
            .iter()
            .map(|id| scopes.get(*id).map(|scope| (*id, scope.state())))
            .collect::<Option<Vec<_>>>();
        *memo = dependencies.map(|origins| State {
            frame: frame.state(),
            origins,
        });
    }
}

#[cfg(test)]
mod tests;

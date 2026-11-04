use super::{Engine, State, Value};
use crate::codebase::postgres::embedded::helpers::program::Expr;

impl Engine<'_> {
    pub(super) fn raise(
        &mut self,
        args: &[Expr],
        state: State,
        at: super::super::Span,
    ) -> Option<Vec<(State, Value)>> {
        let [argument] = args else {
            return None;
        };
        if !self.effects || self.site.is_some() {
            // Exact sinks exclude proven throws; see docs/postgres-facts/low-helper-recovery.md.
            if !self.catalog.value_throw(self.file()?, at) {
                self.failures += 1;
            }
            return Some(Vec::new());
        }
        for (state, value) in self.expression(argument, state)? {
            // An exception must not smuggle a supplied executor back to a caller.
            if state.contains_executor(&value) {
                self.failures += 1;
            }
        }
        Some(Vec::new())
    }
}

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
        if !self.effects {
            // This mode proves returned SQL only. An abrupt path never supplies
            // an operand to the requesting executor, regardless of its effects.
            // Transport proofs separately account for those effects below.
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

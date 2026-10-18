mod dependencies;
mod imports;
mod memo;
mod state;
mod values;
use super::{calls::scopes, Environment, Evaluator, Value};
use crate::fx::FxHashMap;
use std::path::{Path, PathBuf};

#[derive(Default)]
struct CallbackState {
    objects: FxHashMap<u64, (u8, Vec<Value>)>,
    functions: FxHashMap<CallbackIdentity, (u8, Value, std::sync::Arc<state::Snapshot>)>,
    memo: memo::Memo,
}

pub(in crate::codebase::postgres::query_annotation) type CallbackIdentity =
    (PathBuf, u32, Environment, Vec<String>);

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn opaque_callbacks_with_roots(&mut self, arguments: &[Value], depth: u8) {
        let outermost = self.active_callback_functions.is_none();
        self.set_transaction_flag(super::transactions::Cell::CallbackEnabled, true);
        self.set_transaction_flag(super::transactions::Cell::ExecutionsEnabled, true);
        let mut state = CallbackState::default();
        self.callback_values(arguments, depth, &mut state);
        // Only objects reached by this consumer participate. Each revisit
        // spends the existing depth budget, including cyclic installers.
        for revisit_depth in (0..depth).rev() {
            let mut changed = state
                .objects
                .iter()
                .filter_map(|(id, (budget, previous))| {
                    let live = self.live_callbacks(*id);
                    (live != *previous).then_some((*id, (*budget).min(revisit_depth), live))
                })
                .collect::<Vec<_>>();
            // All snapshots remain live while an earlier callback can replace
            // the parent slots from which a later snapshot was captured.
            for (_, _, live) in &changed {
                self.retain_temporary_values(live);
            }
            let functions = state
                .functions
                .iter()
                .map(|(key, (budget, value, before))| {
                    (key.clone(), *budget, value.clone(), before.clone())
                })
                .collect::<Vec<_>>();
            let mut comparison = memo::Memo::default();
            let mut callbacks = Vec::new();
            for (key, budget, value, before) in functions {
                if let Value::Function(function, path, captured) = &value {
                    let current = self.memoized_callback_snapshot(
                        path,
                        *captured,
                        function,
                        budget,
                        &mut comparison,
                    );
                    if current != before {
                        callbacks.push((key, budget.min(revisit_depth), value));
                    }
                }
            }
            if changed.is_empty() && callbacks.is_empty() {
                break;
            }
            callbacks.sort_unstable_by(|left, right| left.0.cmp(&right.0));
            for (key, budget, value) in callbacks {
                self.set_transaction_flag(super::transactions::Cell::CallbackSeen(key), false);
                self.callback_values(std::slice::from_ref(&value), budget, &mut state);
            }
            changed.sort_unstable_by_key(|(id, _, _)| *id);
            for (id, budget, live) in changed {
                state.objects.get_mut(&id).unwrap().1 = live.clone();
                self.callback_values(&live, budget, &mut state);
            }
        }
        if outermost {
            self.set_transaction_flag(super::transactions::Cell::CallbackEnabled, false);
            self.set_transaction_flag(super::transactions::Cell::ExecutionsEnabled, false);
        }
    }
}

#[cfg(test)]
#[path = "callbacks/tests.rs"]
mod tests;

#[cfg(test)]
#[path = "callbacks/dependencies_tests.rs"]
mod dependencies_tests;
#[cfg(test)]
#[path = "callbacks/profile_tests.rs"]
mod profile_tests;

#[cfg(test)]
#[path = "callbacks/imported_tests.rs"]
mod imported_tests;

#[cfg(test)]
#[path = "callbacks/memo_tests.rs"]
mod memo_tests;

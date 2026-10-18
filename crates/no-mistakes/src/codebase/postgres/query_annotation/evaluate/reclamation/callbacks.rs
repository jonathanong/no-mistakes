//! Callback membership never roots frames, but dead-frame keys must be removed promptly.
use super::super::{transactions::Saved, Evaluator};
use super::*;
use std::path::{Path, PathBuf};
impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn note_callback_reference(&mut self, cell: &Cell) -> bool {
        let env = match cell {
            Cell::CallbackSeen(key) | Cell::CallbackRunning(key) => key.2,
            Cell::CallbackEnabled => {
                if self.active_callback_functions.is_none() {
                    self.transactions.reclamation.callback_keys.clear();
                }
                return true;
            }
            _ => return false,
        };
        if self.read_cell(cell) == Saved::Flag(true) {
            self.transactions
                .reclamation
                .callback_keys
                .entry(env)
                .or_default()
                .insert(cell.clone());
        } else if let Some(keys) = self.transactions.reclamation.callback_keys.get_mut(&env) {
            keys.remove(cell);
            if keys.is_empty() {
                self.transactions.reclamation.callback_keys.remove(&env);
            }
        }
        true
    }
    pub(in super::super) fn drop_frame_callback_keys(&mut self, env: Environment) {
        if let Some(keys) = self.transactions.reclamation.callback_keys.remove(&env) {
            for cell in keys {
                match cell {
                    Cell::CallbackSeen(key) => {
                        if let Some(seen) = &mut self.active_callback_functions {
                            seen.remove(&key);
                        }
                    }
                    Cell::CallbackRunning(key) => {
                        if let Some(running) = &mut self.active_callback_executions {
                            running.remove(&key);
                        }
                    }
                    _ => unreachable!("frame callback membership"),
                }
            }
        }
    }
}

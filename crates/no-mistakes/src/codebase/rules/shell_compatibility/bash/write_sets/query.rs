use super::WriteSets;
use crate::fx::FxHashSet;

impl WriteSets {
    pub fn touches(&self, index: usize) -> &FxHashSet<String> {
        self.touches_by_scope.get(&index).unwrap_or(&self.empty)
    }

    pub fn empties(&self, index: usize) -> &FxHashSet<String> {
        self.empties_by_scope.get(&index).unwrap_or(&self.empty)
    }

    pub fn ci_empties(&self, index: usize) -> &FxHashSet<String> {
        self.ci_empties_by_scope.get(&index).unwrap_or(&self.empty)
    }

    pub fn locals(&self, index: usize) -> &FxHashSet<String> {
        self.locals_by_scope.get(&index).unwrap_or(&self.empty)
    }

    pub fn call_touches(&self, index: usize) -> &FxHashSet<String> {
        self.calls
            .get(&index)
            .map_or(&self.empty, |(function, _, _)| self.touches(*function))
    }

    pub fn call_empties(&self, index: usize) -> &FxHashSet<String> {
        self.calls
            .get(&index)
            .map_or(&self.empty, |(function, _, _)| self.empties(*function))
    }

    pub fn call_ci_empties(&self, index: usize) -> &FxHashSet<String> {
        self.calls
            .get(&index)
            .map_or(&self.empty, |(function, _, _)| self.ci_empties(*function))
    }

    pub fn call_dynamic_unset(&self, index: usize) -> bool {
        self.calls
            .get(&index)
            .is_some_and(|(function, _, _)| self.dynamic_unset_scopes.contains(function))
    }

    pub fn dynamic_unset_after(&self, fn_index: usize) -> bool {
        let limit = self.last_call(fn_index);
        self.dynamic_unset_positions
            .iter()
            .any(|&pos| pos == usize::MAX || (pos > fn_index && pos <= limit))
    }

    pub fn called_function(&self, index: usize) -> Option<&str> {
        self.calls.get(&index).map(|(_, name, _)| name.as_str())
    }

    pub fn call_is_callback(&self, index: usize) -> bool {
        self.calls.get(&index).is_some_and(|(_, _, is_cb)| *is_cb)
    }

    pub fn is_known_function(&self, name: &str) -> bool {
        self.all_functions.contains(name)
    }

    pub fn has_dynamic_unset(&self, fn_index: usize) -> bool {
        self.dynamic_unset_scopes.contains(&fn_index)
    }

    fn last_call(&self, fn_index: usize) -> usize {
        self.last_call_by_fn
            .get(&fn_index)
            .copied()
            .unwrap_or(usize::MAX)
    }

    pub fn emptied_after(&self, name: &str, fn_index: usize) -> bool {
        let limit = self.last_call(fn_index);
        let in_range = |positions: &Vec<usize>| {
            positions
                .iter()
                .any(|&pos| pos == usize::MAX || (pos > fn_index && pos <= limit))
        };
        self.empty_positions.get(name).is_some_and(in_range)
            || self
                .ci_empty_positions
                .get(&name.to_ascii_lowercase())
                .is_some_and(in_range)
    }

    pub fn touched_after(&self, name: &str, fn_index: usize) -> bool {
        if self.dynamic_touches.contains(name) {
            return true;
        }
        let limit = self.last_call(fn_index);
        self.touches
            .get(name)
            .is_some_and(|indices| indices.iter().any(|&pos| pos > fn_index && pos <= limit))
    }

    pub fn touched_in_range(&self, name: &str, start: usize, end: usize) -> bool {
        self.range_touches
            .get(name)
            .is_some_and(|indices| indices.iter().any(|&i| i >= start && i <= end))
    }
}

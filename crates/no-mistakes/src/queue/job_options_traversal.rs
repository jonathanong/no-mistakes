use crate::fx::FxHashSet;

#[derive(Default)]
pub(super) struct DeclarationTraversal {
    active: FxHashSet<u32>,
    cycle_count: usize,
}

impl DeclarationTraversal {
    pub(super) fn enter(&mut self, declaration: u32) -> Option<usize> {
        if self.active.insert(declaration) {
            return Some(self.cycle_count);
        }
        self.cycle_count += 1;
        None
    }

    pub(super) fn complete(&mut self, declaration: u32, cycle_count_on_entry: usize) -> bool {
        self.active.remove(&declaration);
        self.cycle_count == cycle_count_on_entry
    }
}

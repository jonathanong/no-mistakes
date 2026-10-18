use super::super::{transactions::Saved, Evaluator};
use super::*;
use std::path::{Path, PathBuf};
impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(in super::super) fn note_reference_cell(&mut self, cell: &Cell) {
        if self.note_callback_reference(cell) {
            return;
        }
        let value = match cell {
            Cell::Binding(env, name) => {
                Some(self.scopes.get(*env).and_then(|scope| scope.get(name)))
            }
            Cell::Dense(id, index) => Some(
                self.argument_objects
                    .get(id)
                    .and_then(|slots| slots.get(*index)),
            ),
            Cell::Extra(id, index) => Some(
                self.argument_extra_slots
                    .get(id)
                    .and_then(|slots| slots.get(index)),
            ),
            _ => None,
        };
        if let Some(value) = value {
            let references = value.map(References::value).unwrap_or_default();
            self.transactions
                .reclamation
                .set(Location::Cell(cell.clone()), references);
            return;
        }
        let id = match cell {
            Cell::Invalidated(id)
            | Cell::Deleted(id, _)
            | Cell::Definite(id, _)
            | Cell::Disconnected(id, _)
            | Cell::Recreated(id, _) => *id,
            Cell::Builder(id) => {
                self.transactions.reclamation.builder_candidates.insert(*id);
                return;
            }
            Cell::Length(id) => {
                self.transactions
                    .reclamation
                    .remove_truncated(*id, self.argument_objects[id].len());
                return;
            }
            _ => return,
        };
        self.transactions.reclamation.candidates.insert(id);
        self.transactions.reclamation.builder_candidates.insert(id);
        if let Saved::Flag(true) = self.read_cell(cell) {
            self.transactions
                .reclamation
                .masks
                .entry(id)
                .or_default()
                .insert(cell.clone());
        } else if let Some(masks) = self.transactions.reclamation.masks.get_mut(&id) {
            masks.remove(cell);
            if masks.is_empty() {
                self.transactions.reclamation.masks.remove(&id);
            }
        }
    }
    pub(in super::super) fn register_frame_references(&mut self, env: Environment) {
        let references = self.scopes[env]
            .iter()
            .filter_map(|(name, value)| {
                let references = References::value(value);
                (!references.arguments.is_empty() || !references.builders.is_empty())
                    .then(|| (Location::Cell(Cell::Binding(env, name.clone())), references))
            })
            .collect::<Vec<_>>();
        for (location, references) in references {
            self.transactions.reclamation.set(location, references);
        }
    }
    pub(in super::super) fn register_argument_references(&mut self, id: u64) {
        self.transactions.reclamation.candidates.insert(id);
        self.transactions
            .reclamation
            .remove_owner(Owner::Argument(id));
        let mut references = Vec::new();
        if let Some(slots) = self.argument_objects.get(&id) {
            references.extend(slots.iter().enumerate().filter_map(|(index, value)| {
                let references = References::value(value);
                (!references.arguments.is_empty() || !references.builders.is_empty())
                    .then_some((Location::Cell(Cell::Dense(id, index)), references))
            }));
        }
        if let Some(slots) = self.argument_extra_slots.get(&id) {
            references.extend(slots.iter().filter_map(|(index, value)| {
                let references = References::value(value);
                (!references.arguments.is_empty() || !references.builders.is_empty())
                    .then_some((Location::Cell(Cell::Extra(id, *index)), references))
            }));
        }
        for (location, references) in references {
            self.transactions.reclamation.set(location, references);
        }
    }
    pub(in super::super) fn register_mapping_references(&mut self, env: Environment) {
        for (id, _) in self.mapped_arguments.get(&env).into_iter().flatten() {
            let references = References {
                arguments: FxHashSet::from_iter([*id]),
                builders: Default::default(),
            };
            self.transactions
                .reclamation
                .set(Location::Mapping(env, *id), references);
        }
    }
    pub(in super::super) fn builder_locations(&self, ids: &FxHashSet<u64>) -> Vec<Cell> {
        self.transactions.reclamation.builder_locations(ids)
    }
    pub(in super::super) fn drop_frame_references(&mut self, env: Environment) {
        self.transactions
            .reclamation
            .remove_owner(Owner::Frame(env));
    }
}

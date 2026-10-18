use super::*;
fn decrement(map: &mut FxHashMap<u64, usize>, id: u64) {
    let count = map.get_mut(&id).expect("registered incoming reference");
    *count -= 1;
    if *count == 0 {
        map.remove(&id);
    }
}
impl Reclamation {
    pub(super) fn set(&mut self, location: Location, next: References) {
        if self.locations.get(&location) == Some(&next) {
            return;
        }
        self.remove(&location);
        if next.arguments.is_empty() && next.builders.is_empty() {
            return;
        }
        let owner = location.owner();
        for id in &next.arguments {
            *self.incoming.entry(*id).or_default() += 1;
            if matches!(owner, Owner::Frame(_)) {
                *self.roots.entry(*id).or_default() += 1;
            }
        }
        if let Location::Cell(cell) = &location {
            for id in &next.builders {
                self.holders.entry(*id).or_default().insert(cell.clone());
            }
        }
        self.record_reference_insert(&next);
        self.owners
            .entry(owner)
            .or_default()
            .insert(location.clone());
        self.locations.insert(location, next);
    }
    pub(super) fn remove(&mut self, location: &Location) {
        let Some(old) = self.locations.remove(location) else {
            return;
        };
        let owner = location.owner();
        for id in old.arguments {
            decrement(&mut self.incoming, id);
            if matches!(owner, Owner::Frame(_)) {
                decrement(&mut self.roots, id);
            }
            self.candidates.insert(id);
        }
        if let Location::Cell(cell) = location {
            for id in old.builders {
                let holders = self
                    .holders
                    .get_mut(&id)
                    .expect("registered builder holder");
                holders.remove(cell);
                if holders.is_empty() {
                    self.holders.remove(&id);
                }
                self.builder_candidates.insert(id);
            }
        }
        let locations = self
            .owners
            .get_mut(&owner)
            .expect("registered reference owner");
        locations.remove(location);
        if locations.is_empty() {
            self.owners.remove(&owner);
        }
    }
    pub(super) fn remove_owner(&mut self, owner: Owner) {
        let locations = self
            .owners
            .get(&owner)
            .map(|set| set.iter().cloned().collect::<Vec<_>>())
            .unwrap_or_default();
        for location in locations {
            self.remove(&location);
        }
    }
    pub(super) fn targets(&self, id: u64) -> impl Iterator<Item = u64> + '_ {
        self.owners
            .get(&Owner::Argument(id))
            .into_iter()
            .flatten()
            .flat_map(move |location| self.locations[location].arguments.iter().copied())
    }
    pub(super) fn remove_truncated(&mut self, id: u64, length: usize) {
        let locations = self.owners.get(&Owner::Argument(id)).into_iter().flatten()
            .filter(|location| matches!(location, Location::Cell(Cell::Dense(_, index)) if *index >= length))
            .cloned().collect::<Vec<_>>();
        for location in locations {
            self.remove(&location);
        }
    }
    pub(super) fn pin(&mut self, id: u64) {
        *self.pins.entry(id).or_default() += 1;
    }
    pub(super) fn unpin(&mut self, id: u64) {
        let count = self.pins.get_mut(&id).expect("registered identity pin");
        *count -= 1;
        if *count == 0 {
            self.pins.remove(&id);
        }
        self.candidates.insert(id);
        self.builder_candidates.insert(id);
    }
    pub(super) fn builder_locations(&self, ids: &FxHashSet<u64>) -> Vec<Cell> {
        let mut cells = ids
            .iter()
            .filter_map(|id| self.holders.get(id))
            .flatten()
            .cloned()
            .collect::<FxHashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        // Replacement order must not depend on hash-table insertion history.
        cells.sort_unstable_by(|left, right| holder_key(left).cmp(&holder_key(right)));
        cells
    }
}
fn holder_key(cell: &Cell) -> (u8, u64, usize, &str) {
    match cell {
        Cell::Binding(env, name) => (0, *env as u64, 0, name),
        Cell::Dense(id, index) => (1, *id, *index, ""),
        Cell::Extra(id, index) => (2, *id, *index, ""),
        _ => unreachable!("only value-bearing cells own builder references"),
    }
}

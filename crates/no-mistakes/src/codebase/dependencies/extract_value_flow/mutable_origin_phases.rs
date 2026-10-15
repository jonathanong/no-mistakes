use super::*;

impl ReceiverBindingOrigins {
    pub(in super::super) fn record_pattern_root(
        &mut self,
        key: (usize, &str),
        catalog: &std::sync::Arc<phase_catalog::PhaseCatalog>,
        phase: u32,
        owner: Option<CallableId>,
        value: ValueExpression,
        guaranteed: bool,
    ) {
        if let Some(root) = self
            .roots
            .get_mut(&key.0)
            .and_then(|names| names.get_mut(key.1))
        {
            let initial = root
                .owners
                .get(&owner)
                .and_then(|states| states.snapshots.before(catalog.start).map(|(_, id)| *id));
            let patterns = root.patterns.get_or_insert_with(Default::default);
            let previous = patterns.previous(catalog, phase, owner).or(initial);
            let leaf = self.arena.len();
            self.arena.push(Origin::Value(value));
            let selected = if !guaranteed {
                previous.map_or(leaf, |previous| {
                    let id = self.arena.len();
                    self.arena.push(Origin::Either(previous, leaf));
                    id
                })
            } else {
                leaf
            };
            patterns.record(catalog, phase, owner, initial, selected, guaranteed);
            root.generation += 1;
        }
    }
    pub(in super::super) fn finish_pattern_root(
        &mut self,
        key: (usize, &str),
        start: u32,
        end: u32,
        owner: Option<CallableId>,
        guaranteed: bool,
    ) {
        let Some(root) = self
            .roots
            .get_mut(&key.0)
            .and_then(|names| names.get_mut(key.1))
        else {
            return;
        };
        let Some((mut selected, initial, retains_initial)) = root
            .patterns
            .as_ref()
            .and_then(|patterns| patterns.final_selected(start, owner))
        else {
            return;
        };
        if !guaranteed && !retains_initial {
            if let Some(initial) = initial.filter(|initial| *initial != selected) {
                let joined = self.arena.len();
                self.arena.push(Origin::Either(initial, selected));
                selected = joined;
            }
        }
        // The final execution version wins over physically later RHS source
        // positions, which run before the LHS writes in an assignment pattern.
        let states = root.owners.entry(owner).or_default();
        #[cfg(feature = "test-instrumentation")]
        states.observe_insertion(end);
        states.snapshots.insert(end, selected);
        // Normal collection already retained every leaf for foreign readers;
        // execution snapshots need no duplicate possible-origin join.
        root.generation += 1;
    }
}

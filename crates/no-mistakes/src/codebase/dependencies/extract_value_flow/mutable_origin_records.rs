use super::*;

impl ReceiverBindingOrigins {
    pub(in super::super) fn record(
        &mut self,
        key: (usize, &str),
        location: (u32, Option<CallableId>),
        value: impl Into<Origin>,
        guaranteed: bool,
    ) {
        let leaf = self.arena.len();
        self.arena.push(value.into());
        let names = self.roots.entry(key.0).or_default();
        if let Some(root) = names.get_mut(key.1) {
            root.record(&mut self.arena, location, leaf, guaranteed);
        } else {
            #[cfg(feature = "test-instrumentation")]
            {
                self.owned_keys += 1;
            }
            let mut root = RootOrigins::default();
            root.record(&mut self.arena, location, leaf, guaranteed);
            names.insert(key.1.to_owned(), root);
        }
    }
}
impl RootOrigins {
    fn record(
        &mut self,
        arena: &mut Vec<Origin>,
        location: (u32, Option<CallableId>),
        leaf: usize,
        guaranteed: bool,
    ) {
        self.generation += 1;
        let states = self.owners.entry(location.1).or_default();
        let selected = if guaranteed {
            leaf
        } else {
            #[cfg(feature = "test-instrumentation")]
            {
                self.predecessor_queries += 1;
            }
            let previous = states.snapshots.before(location.0).map(|(_, id)| *id);
            if let Some(previous) = previous {
                let id = arena.len();
                arena.push(Origin::Either(previous, leaf));
                id
            } else {
                leaf
            }
        };
        #[cfg(feature = "test-instrumentation")]
        states.observe_insertion(location.0);
        states.snapshots.insert(location.0, selected);
        states.possible.push(leaf);
    }
}

#[cfg(feature = "test-instrumentation")]
impl RootStates {
    pub(super) fn observe_insertion(&mut self, offset: u32) {
        let class = match self.snapshots.last_key_value() {
            None => 0,
            Some((previous, _)) if *previous < offset => 0,
            Some((previous, _)) if *previous == offset => 1,
            _ => 2,
        };
        self.insertion_order[class] += 1;
    }
}

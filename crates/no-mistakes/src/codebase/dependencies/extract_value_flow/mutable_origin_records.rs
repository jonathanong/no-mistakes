use super::*;

impl ReceiverBindingOrigins {
    pub(in super::super) fn record(
        &mut self,
        key: (usize, &str),
        location: (u32, Option<CallableId>),
        value: ValueExpression,
        guaranteed: bool,
    ) {
        let leaf = self.arena.len();
        self.arena.push(Origin::Value(value));
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
        let previous = states
            .snapshots
            .range(..location.0)
            .next_back()
            .map(|(_, id)| *id);
        let selected = match (guaranteed, previous) {
            (false, Some(previous)) => {
                let id = arena.len();
                arena.push(Origin::Either(previous, leaf));
                id
            }
            _ => leaf,
        };
        states.snapshots.insert(location.0, selected);
        states.possible.push(leaf);
    }
}

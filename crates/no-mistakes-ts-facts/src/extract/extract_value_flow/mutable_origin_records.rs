use super::*;

impl ReceiverBindingOrigins {
    #[cfg(feature = "test-instrumentation")]
    pub(in super::super) fn record_counts(&self, facts: &mut ValueFlowFacts) {
        #[cfg(all(test, feature = "test-instrumentation"))]
        fusion_tests::observe_owners(&self.roots);
        facts.receiver_only_nodes = self.arena.len();
        facts.module_provider_work = self.module_work.get();
        facts.receiver_only_lookups = self.lookups.get();
        facts.receiver_only_visits = self.visits.get();
        facts.receiver_only_pending_spills = self.spills.get();
        facts.receiver_only_membership_probes = self.membership_probes.get();
        facts.receiver_only_owned_keys = self.owned_keys;
        facts.receiver_only_snapshot_queries = self.snapshot_queries.get()[0];
        facts.receiver_only_snapshot_work = self.snapshot_queries.get()[1..].try_into().unwrap();
        for states in self
            .roots
            .values()
            .flat_map(|names| names.values())
            .flat_map(|root| root.owners.values())
        {
            facts.receiver_only_snapshot_tree_buffers += states.snapshots.tree_buffer();
            for (total, count) in facts
                .receiver_only_insertion_order
                .iter_mut()
                .zip(states.insertion_order)
            {
                *total += count;
            }
        }
        facts.receiver_only_predecessor_queries = self
            .roots
            .values()
            .flat_map(|names| names.values())
            .map(|root| root.predecessor_queries)
            .sum();
    }

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

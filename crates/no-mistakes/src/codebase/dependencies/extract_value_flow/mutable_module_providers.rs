use super::*;
use std::sync::Arc;

impl ReceiverBindingOrigins {
    pub(super) fn module_provider_index(
        &self,
        key: &(usize, String),
        path: &[Option<String>],
    ) -> Vec<(nested::ReceiverWrite, Arc<ModuleProviderTarget>)> {
        let Some(root) = self.roots.get(&key.0).and_then(|names| names.get(&key.1)) else {
            return Vec::new();
        };
        let Some(states) = root.owners.get(&None) else {
            return Vec::new();
        };
        let uncertain = root.patterns.is_some() || states.snapshots.unordered();
        #[cfg(feature = "test-instrumentation")]
        self.module_work.set([
            self.module_work.get()[0] + 1,
            self.module_work.get()[1],
            self.module_work.get()[2],
        ]);
        let mut targets = fx_map();
        let reads = vec![None; path.len()];
        let mut append = |id, start, end| {
            #[cfg(feature = "test-instrumentation")]
            self.module_work.set([
                self.module_work.get()[0],
                self.module_work.get()[1] + 1,
                self.module_work.get()[2],
            ]);
            let mut references = Vec::new();
            match &self.arena[id] {
                Origin::Value(value) => {
                    nested::write_receiver_references(value, path, &reads, &mut references)
                }
                Origin::SingleNamedField(value) => {
                    self.append_single(value, path, &reads, &mut references)
                }
                Origin::Either(..) => return,
            }
            for (receiver, mut reads, mut read) in references {
                if !self.contains(receiver.0 .0, &receiver.0 .1) {
                    reads.fill(None);
                    read = (0, None);
                }
                let target: &mut ModuleProviderTarget =
                    targets.entry((receiver, reads, read)).or_default();
                target.uncertain = uncertain;
                target.intervals.push((start, end));
            }
        };
        if uncertain {
            for id in states.possible.before(usize::MAX) {
                append(id, 0, u32::MAX);
            }
        } else {
            let mut end = u32::MAX;
            states
                .snapshots
                .reversed(|start, selected| match &self.arena[selected] {
                    Origin::Either(_, leaf) => append(*leaf, start, end),
                    _ => {
                        append(selected, start, end);
                        end = start;
                    }
                });
        }
        let mut indexed: Vec<_> = targets
            .into_iter()
            .map(|(reference, mut target)| {
                target.finish();
                (reference, Arc::new(target))
            })
            .collect();
        indexed.sort_by(|(left, _), (right, _)| left.cmp(right));
        #[cfg(feature = "test-instrumentation")]
        self.module_work.set([
            self.module_work.get()[0],
            self.module_work.get()[1],
            self.module_work.get()[2] + indexed.len(),
        ]);
        indexed
    }
}

use super::*;

/// Latest provider from two distinct owners answers every foreign-owner suffix
/// query without replaying bodies for each writer or each collection watermark.
struct Providers {
    newest: (usize, CallableId),
    other: Option<(usize, CallableId)>,
}
impl Providers {
    fn insert(&mut self, id: usize, owner: CallableId) {
        if self.newest.1 == owner {
            self.newest.0 = self.newest.0.max(id);
        } else if id > self.newest.0 {
            self.other = Some(self.newest);
            self.newest = (id, owner);
        } else if self.other.is_none_or(|other| id > other.0) {
            self.other = Some((id, owner));
        }
    }
    fn after(&self, writer: CallableId, before: usize) -> bool {
        let selected = if self.newest.1 != writer {
            Some(self.newest)
        } else {
            self.other
        };
        selected.is_some_and(|(id, _)| id >= before)
    }
}

#[derive(Default)]
pub(super) struct ForeignProviderIndex {
    references: Vec<(nested::ReceiverWrite, Providers)>,
    pub(super) module_references:
        Vec<(nested::ReceiverWrite, std::sync::Arc<ModuleProviderTarget>)>,
}
impl ForeignProviderIndex {
    pub(super) fn references(
        &self,
        writer: Option<CallableId>,
        before: usize,
    ) -> Vec<nested::ReceiverWrite> {
        self.references
            .iter()
            .filter(|(_, providers)| {
                // Member summaries (MAX) mutate the acquired object itself;
                // only root rebindings exclude a saved acquisition's owner.
                providers.newest.0 == usize::MAX
                    || writer.map_or(providers.newest.0 >= before, |writer| {
                        providers.after(writer, before)
                    })
            })
            .map(|(reference, _)| reference.clone())
            .collect()
    }
}

impl ReceiverBindingOrigins {
    pub(super) fn foreign_provider_index(
        &self,
        key: &(usize, String),
        path: &[Option<String>],
        member_references: Vec<(nested::ReceiverWrite, Option<CallableId>, (u32, u32))>,
    ) -> ForeignProviderIndex {
        let root = self.roots.get(&key.0).and_then(|names| names.get(&key.1));
        let reads = vec![None; path.len()];
        let mut indexed = fx_map();
        let mut insert = |(receiver, mut reads, mut read): nested::ReceiverWrite, id, owner| {
            // Immutable imported roots need one conservative identity.
            // Mutable roots keep their distinct acquisition coordinates.
            if !self.contains(receiver.0 .0, &receiver.0 .1) {
                reads.fill(None);
                read = (0, None);
            }
            indexed
                .entry((receiver, reads, read))
                .and_modify(|providers: &mut Providers| providers.insert(id, owner))
                .or_insert(Providers {
                    newest: (id, owner),
                    other: None,
                });
        };
        for (owner, states) in root.into_iter().flat_map(|root| &root.owners) {
            let Some(owner) = owner else { continue };
            for id in states.possible.before(usize::MAX) {
                #[cfg(feature = "test-instrumentation")]
                foreign::count(2, 1);
                let mut references = Vec::new();
                match &self.arena[id] {
                    Origin::Value(value) => {
                        nested::write_receiver_references(value, path, &reads, &mut references)
                    }
                    Origin::SingleNamedField(value) => {
                        self.append_single(value, path, &reads, &mut references)
                    }
                    Origin::Either(..) => unreachable!("possible origins contain leaves only"),
                }
                for reference in references {
                    insert(reference, id, *owner);
                }
            }
        }
        let mut module_references = self.module_provider_index(key, path);
        // Callable stores are summaries; module stores retain execution intervals.
        for (reference, owner, interval) in member_references {
            if let Some(owner) = owner {
                insert(reference, usize::MAX, owner);
            } else {
                module_references.push((
                    reference,
                    std::sync::Arc::new(ModuleProviderTarget {
                        intervals: vec![interval],
                        uncertain: false,
                    }),
                ));
            }
        }
        let mut references: Vec<_> = indexed.into_iter().collect();
        references.sort_by(|(left, _), (right, _)| left.cmp(right));
        #[cfg(feature = "test-instrumentation")]
        foreign::count(3, references.len());
        ForeignProviderIndex {
            references,
            module_references,
        }
    }
}

#[cfg(test)]
#[path = "mutable_foreign_providers_tests.rs"]
mod tests;

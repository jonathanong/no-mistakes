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
        writer: CallableId,
        before: usize,
    ) -> Vec<nested::ReceiverWrite> {
        self.references
            .iter()
            .filter(|(_, providers)| providers.after(writer, before))
            .map(|(reference, _)| reference.clone())
            .collect()
    }
}

impl ReceiverBindingOrigins {
    pub(super) fn foreign_provider_index(
        &self,
        key: &(usize, String),
        path: &[Option<String>],
    ) -> ForeignProviderIndex {
        let Some(root) = self.roots.get(&key.0).and_then(|names| names.get(&key.1)) else {
            return ForeignProviderIndex::default();
        };
        let reads = vec![None; path.len()];
        let mut indexed = fx_map();
        for (owner, states) in &root.owners {
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
                for (receiver, mut reads, mut read) in references {
                    // Immutable imported roots need one conservative identity.
                    // Mutable roots keep their distinct acquisition coordinates.
                    if !self.contains(receiver.0 .0, &receiver.0 .1) {
                        reads.fill(None);
                        read = (0, None);
                    }
                    indexed
                        .entry((receiver, reads, read))
                        .and_modify(|providers: &mut Providers| providers.insert(id, *owner))
                        .or_insert(Providers {
                            newest: (id, *owner),
                            other: None,
                        });
                }
            }
        }
        let mut references: Vec<_> = indexed.into_iter().collect();
        references.sort_by(|(left, _), (right, _)| left.cmp(right));
        #[cfg(feature = "test-instrumentation")]
        foreign::count(3, references.len());
        ForeignProviderIndex {
            references,
            module_references: self.module_provider_index(key, path),
        }
    }
}

#[cfg(test)]
#[path = "mutable_foreign_providers_tests.rs"]
mod tests;

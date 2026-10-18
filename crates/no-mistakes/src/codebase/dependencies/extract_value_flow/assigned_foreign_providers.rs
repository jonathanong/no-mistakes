use super::*;

impl AssignedReceiverOrigins {
    /// Callable member stores can supply a later receiver acquisition even when
    /// their bodies were visited after the writer. Reuse canonical RHS timelines.
    pub(in super::super) fn foreign_references(
        &self,
        root: usize,
        path: &[Option<String>],
    ) -> Vec<(nested::ReceiverWrite, CallableId)> {
        let mut output = Vec::new();
        let reads = vec![None; path.len()];
        for prefixes in self
            .roots
            .get(&root)
            .into_iter()
            .flat_map(|versions| versions.values())
        {
            for (prefix, owners) in prefixes {
                if prefix.len() > path.len()
                    || !prefix.iter().zip(path).all(|(stored, requested)| {
                        stored.is_none() || requested.is_none() || stored == requested
                    })
                {
                    continue;
                }
                for (owner, identities) in owners {
                    let Some(owner) = owner else { continue };
                    for timeline in identities.values() {
                        for origin in timeline.events.values() {
                            let mut references = Vec::new();
                            nested::write_receiver_references(
                                &origin.value,
                                &path[prefix.len()..],
                                &reads[prefix.len()..],
                                &mut references,
                            );
                            output.extend(
                                references.into_iter().map(|reference| (reference, *owner)),
                            );
                        }
                    }
                }
            }
        }
        output
    }
}

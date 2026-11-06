use super::*;

impl AssignedReceiverOrigins {
    /// Callable member stores can supply a later receiver acquisition even when
    /// their bodies were visited after the writer. Reuse canonical RHS timelines.
    pub(in super::super) fn foreign_references(
        &self,
        root: usize,
        path: &[Option<String>],
        budget: &mut mutable_origins::WorkBudget,
    ) -> Option<nested::ForeignMemberReferences> {
        let mut output = Vec::new();
        let reads = vec![None; path.len()];
        for prefixes in self
            .roots
            .get(&root)
            .into_iter()
            .flat_map(|versions| versions.values())
        {
            for (prefix, owners) in prefixes {
                budget.charge(1 + prefix.len())?;
                if prefix.len() > path.len()
                    || !prefix.iter().zip(path).all(|(stored, requested)| {
                        stored.is_none() || requested.is_none() || stored == requested
                    })
                {
                    continue;
                }
                for (owner, identities) in owners {
                    for timeline in identities.values() {
                        let mut end = u32::MAX;
                        // A complete store ends earlier module providers. Visit
                        // each event once, including long runs of partial stores.
                        for (at, origin) in timeline.events.iter().rev() {
                            budget.charge(1)?;
                            let mut references = budget.output();
                            nested::write_receiver_references(
                                &origin.value,
                                &path[prefix.len()..],
                                &reads[prefix.len()..],
                                &mut references,
                            );
                            output.extend(
                                references
                                    .finish()?
                                    .into_iter()
                                    .map(|reference| (reference, *owner, (*at, end))),
                            );
                            if owner.is_none() && origin.complete {
                                end = *at;
                            }
                        }
                    }
                }
            }
        }
        Some(output)
    }
}

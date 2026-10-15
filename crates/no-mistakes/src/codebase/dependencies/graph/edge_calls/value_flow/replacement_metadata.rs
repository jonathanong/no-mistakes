type ReplacementMutations = FxHashMap<usize, (FxHashSet<String>, bool)>;
type UnsupportedMutations = FxHashMap<String, (FxHashSet<String>, bool)>;

fn replacement_mutations(
    file: &CallableFileIndex,
    receivers: &FxHashMap<String, usize>,
    writes: &index::PreparedReplacements,
) -> (ReplacementMutations, UnsupportedMutations) {
    let repeat_writer = |offset, context: Option<CallableId>, multiple| {
        multiple
            || context.is_some()
            || writes
                .region_at(context, offset)
                .is_some_and(|id| writes.facts.regions[id].may_repeat)
    };
    let mut mutations = fx_map();
    let mut unsupported: FxHashMap<String, (FxHashSet<String>, bool)> = fx_map();
    for write in &writes.facts.writes {
        if !receivers.contains_key(&write.receiver) {
            let (properties, dynamic) = unsupported.entry(write.receiver.clone()).or_default();
            if let Some(property) = &write.property {
                properties.insert(property.clone());
            } else {
                *dynamic = true;
            }
        }
    }
    if let Some(histories) = &file.values.receiver_mutations {
        if let Some(names) = histories.get(&0) {
            for name in file.imported.keys() {
                if !receivers.contains_key(name) {
                    if let Some(history) = names.get(name) {
                        let (properties, dynamic) = unsupported.entry(name.clone()).or_default();
                        if let Some(assigned) = &history.assigned {
                            properties.extend(assigned.members.keys().cloned());
                            *dynamic |= assigned.dynamic.is_some();
                        }
                    }
                }
            }
            for (name, receiver) in receivers {
                if let Some(history) = names.get(name) {
                    let (properties, dynamic) = mutations
                        .entry(*receiver)
                        .or_insert_with(|| (fx_set(), false));
                    if let Some(assigned) = &history.assigned {
                        properties.extend(
                            assigned
                                .members
                                .iter()
                                .filter(|(_, write)| {
                                    repeat_writer(
                                        write.offset,
                                        write.context,
                                        write.multiple_contexts,
                                    )
                                })
                                .map(|(property, _)| property.clone()),
                        );
                        *dynamic |= assigned.dynamic.as_ref().is_some_and(|write| {
                            repeat_writer(write.offset, write.context, write.multiple_contexts)
                        });
                    }
                    *dynamic |= history
                        .dynamic
                        .as_ref()
                        .and_then(|history| history.summary.as_ref())
                        .is_some_and(|summary| {
                            summary
                                .contexts
                                .iter()
                                .any(|context| repeat_writer(summary.offset, *context, false))
                        });
                }
            }
        }
    }
    (mutations, unsupported)
}

fn replacement_snapshot_owners(
    file: &CallableFileIndex,
    sites: &[&ResolvedCallSite],
) -> FxHashMap<CallableId, (bool, bool)> {
    let mut snapshot_owners: FxHashMap<_, _> = file
        .values
        .returns
        .iter()
        .filter(|(owner, returns)| {
            returns.len() == 1
                && !file.values.async_callables.contains(owner)
                && !file.values.generator_callables.contains(owner)
        })
        .map(|(owner, _)| {
            (
                *owner,
                (
                    !file
                        .values
                        .receiver_mutations
                        .as_ref()
                        .is_some_and(|histories| histories.writer_contexts.contains(owner)),
                    true,
                ),
            )
        })
        .collect();
    for site in sites {
        if let Some(owner) = site.caller_id {
            if let Some((_, closed)) = snapshot_owners.get_mut(&owner) {
                *closed = false;
            }
        }
    }
    snapshot_owners
}

use super::*;

#[path = "replacement_index.rs"]
mod index;
#[path = "replacement_owners.rs"]
mod owners;
#[path = "replacement_payloads.rs"]
mod payloads;
#[path = "replacement_receivers.rs"]
mod receivers;
#[path = "replacement_writers.rs"]
mod writers;

/// One immutable request projection; ordinary files do not create an index.
pub(super) struct ReplacementCatalog {
    files: FxHashMap<std::path::PathBuf, ReplacementFile>,
    mutated_external: FxHashMap<String, FxHashMap<String, (FxHashSet<String>, bool)>>,
    #[cfg(feature = "test-instrumentation")]
    probe: ReplacementProbeSnapshot,
}
pub(super) struct ReplacementFile {
    receivers: FxHashMap<String, usize>,
    writes: index::PreparedReplacements,
    repeated_owners: FxHashSet<CallableId>,
    payloads: payloads::ReplacementPayloads,
    snapshot_owners: FxHashSet<CallableId>,
    mutations: FxHashMap<usize, (FxHashSet<String>, bool)>,
    unsupported: FxHashMap<String, (FxHashSet<String>, bool)>,
    #[cfg(feature = "test-instrumentation")]
    lookups: ReplacementLookupProbe,
}
impl ReplacementCatalog {
    pub(super) fn prepare(
        inputs: &GraphEdgeBuildInputs<'_>,
        facts: &dyn TsFactLookup,
        resolver: &dyn ImportResolution,
        indexes: &CallableResolutionIndexes,
        pass: &CallPass,
        incoming: &ParameterInputs,
    ) -> Option<Self> {
        if pass.replacement_files.is_empty() {
            return None;
        }
        // Group canonical occurrences once, rather than scanning the whole
        // pass for every hot file or rebuilding ownership evidence per read.
        let mut sites: FxHashMap<_, Vec<_>> = pass
            .replacement_files
            .iter()
            .map(|(path, _)| (path.as_path(), Vec::new()))
            .collect();
        for site in &pass.sites {
            if let Some(file) = sites.get_mut(site.file.as_path()) {
                file.push(site);
            }
        }
        let mut identities = receivers::ReceiverIdentities::default();
        let mut files = fx_map();
        #[cfg(feature = "test-instrumentation")]
        let mut probe = ReplacementProbeSnapshot {
            catalog_builds: 1,
            ..Default::default()
        };
        let mut prepared = Vec::new();
        let mut foreign = writers::ForeignWriters::default();
        for (path, replacements) in &pass.replacement_files {
            let Some(file) = indexes.file(facts, path) else {
                continue;
            };
            let receivers = identities.file(inputs, facts, resolver, indexes, path, &file);
            let repeated_owners = owners::repeated_owners(
                path,
                &file,
                replacements,
                sites.get(path.as_path()).map_or(&[], Vec::as_slice),
                facts
                    .get_ts_facts(path)
                    .map_or(&[], |facts| facts.symbol_references.as_slice()),
            );
            let writes =
                index::PreparedReplacements::new(std::sync::Arc::clone(replacements), &receivers);
            foreign.add(path, &file, &receivers, replacements);
            prepared.push((path, file, receivers, repeated_owners, writes));
        }
        #[cfg(feature = "test-instrumentation")]
        {
            probe.receiver_normalizations = identities.normalizations;
            probe.canonical_receiver_ids = identities.imported_ids.len();
            probe.local_receiver_ids = identities.local_ids();
        }
        let mutated_external = identities.into_mutated_external(&foreign);
        let mut flow = ValueFlowResolution {
            inputs,
            facts,
            resolver,
            indexes,
            incoming,
            supers: &pass.super_constructors,
            replacements: None,
            replacement_read: false,
            foreign_acquisition: false,
            used_parameters: fx_set(),
            parameters_enabled: false,
            initializing: Vec::new(),
            cache: &pass.parameter_cache,
            visiting: fx_set(),
            calls_visiting: fx_set(),
            default_origins: None,
            acquisition_context: None,
            invocation_read: None,
            member_read: None,
        };
        for (path, file, receivers, repeated_owners, writes) in prepared {
            #[allow(unused_mut)]
            let mut payloads = payloads::ReplacementPayloads::prepare(
                &mut flow,
                path,
                &file,
                &receivers,
                &writes,
                &repeated_owners,
                &foreign,
            );
            #[cfg(feature = "test-instrumentation")]
            {
                probe.hot_file_indexes += 1;
                let (writes, regions) = writes.retained_counts();
                probe.retained_write_indices += writes;
                probe.region_segments += regions;
                probe.append(std::mem::take(&mut payloads.probe));
            }
            let mut mutations = fx_map();
            let mut unsupported: FxHashMap<String, (FxHashSet<String>, bool)> = fx_map();
            for write in &writes.facts.writes {
                if !receivers.contains_key(&write.receiver) {
                    let (properties, dynamic) =
                        unsupported.entry(write.receiver.clone()).or_default();
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
                                let (properties, dynamic) =
                                    unsupported.entry(name.clone()).or_default();
                                if let Some(assigned) = &history.assigned {
                                    properties.extend(assigned.members.keys().cloned());
                                    *dynamic |= assigned.dynamic.is_some();
                                }
                            }
                        }
                    }
                    for (name, receiver) in &receivers {
                        if let Some(history) = names.get(name) {
                            let (properties, dynamic) = mutations
                                .entry(*receiver)
                                .or_insert_with(|| (fx_set(), false));
                            if let Some(assigned) = &history.assigned {
                                properties.extend(assigned.members.keys().cloned());
                                *dynamic |= assigned.dynamic.is_some();
                            }
                            *dynamic |= history
                                .dynamic
                                .as_ref()
                                .is_some_and(|history| history.summary.is_some());
                        }
                    }
                }
            }
            let mut snapshot_owners: FxHashSet<_> = file
                .values
                .returns
                .iter()
                .filter(|(owner, returns)| {
                    returns.len() == 1
                        && !file.values.async_callables.contains(owner)
                        && !file.values.generator_callables.contains(owner)
                        && !file
                            .values
                            .receiver_mutations
                            .as_ref()
                            .is_some_and(|histories| histories.writer_contexts.contains(owner))
                })
                .map(|(owner, _)| *owner)
                .collect();
            for site in sites.get(path.as_path()).into_iter().flatten() {
                if let Some(owner) = site.caller_id {
                    snapshot_owners.remove(&owner);
                }
            }
            files.insert(
                path.clone(),
                ReplacementFile {
                    receivers,
                    writes,
                    repeated_owners,
                    payloads,
                    snapshot_owners,
                    mutations,
                    unsupported,
                    #[cfg(feature = "test-instrumentation")]
                    lookups: Default::default(),
                },
            );
        }
        Some(Self {
            files,
            mutated_external,
            #[cfg(feature = "test-instrumentation")]
            probe,
        })
    }
    #[cfg(feature = "test-instrumentation")]
    pub(super) fn snapshot(&self) -> ReplacementProbeSnapshot {
        let mut probe = self.probe.clone();
        for file in self.files.values() {
            probe.lookup_count += file.lookups.count();
            probe.prefix_searches += file.writes.prefix_searches.count();
        }
        probe.rhs.sort_by(|left, right| {
            (&left.file, left.write_index).cmp(&(&right.file, right.write_index))
        });
        probe
    }
    pub(super) fn mutated_member(&self, target: &ResolvedCallTarget, member: &str) -> bool {
        let ResolvedCallTarget::ModuleExport {
            specifier,
            export_path,
            repository_target: None,
            ..
        } = target
        else {
            return false;
        };
        self.mutated_external
            .get(specifier)
            .and_then(|exports| exports.get(export_path))
            .is_some_and(|(properties, dynamic)| *dynamic || properties.contains(member))
    }
    pub(super) fn file(&self, path: &std::path::Path) -> Option<&ReplacementFile> {
        self.files.get(path)
    }
}

impl ReplacementFile {
    pub(super) fn snapshot_owner(&self, owner: CallableId) -> bool {
        self.snapshot_owners.contains(&owner)
    }

    pub(super) fn target(
        &self,
        receiver: &str,
        property: &str,
        offset: u32,
        owner: Option<CallableId>,
    ) -> Option<&ResolvedCallTarget> {
        #[cfg(feature = "test-instrumentation")]
        self.lookups.record(1);
        let source_receiver = receiver;
        let region = self.writes.region_at(owner, offset);
        // Outside any repeat owner/region, ordinary snapshot invalidation owns
        // the answer. A literal zero/one loop does not acquire a backedge.
        if owner.is_none() && region.is_none() {
            return None;
        }
        if region.is_some_and(|id| !self.writes.facts.regions[id].may_repeat) {
            return None;
        }
        let Some(receiver) = self.receivers.get(source_receiver).copied() else {
            return self
                .unsupported
                .get(source_receiver)
                .filter(|(properties, dynamic)| *dynamic || properties.contains(property))
                .map(|_| &ResolvedCallTarget::Unknown);
        };
        if self.writes.reset(receiver) {
            return Some(&ResolvedCallTarget::Unknown);
        }
        let phases = self
            .writes
            .phases(receiver, property, owner, region, offset);
        if phases.is_none() {
            return self
                .mutations
                .get(&receiver)
                .filter(|(properties, dynamic)| *dynamic || properties.contains(property))
                .map(|_| &ResolvedCallTarget::Unknown);
        }
        let phases = phases.expect("prepared phases");
        if region.map_or_else(
            || owner.is_some_and(|owner| self.repeated_owners.contains(&owner)),
            |id| self.writes.facts.regions[id].repeated,
        ) {
            self.payloads
                .target(receiver, property, owner, region, phases.prefix)
        } else {
            Some(&ResolvedCallTarget::Unknown)
        }
    }
}

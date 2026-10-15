use super::*;

#[path = "replacement_calls.rs"]
mod calls;
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
    mutated_external: FxHashMap<String, FxHashMap<String, writers::PreparedWriters>>,
    #[cfg(feature = "test-instrumentation")]
    probe: ReplacementProbeSnapshot,
}
pub(super) struct ReplacementFile {
    receivers: FxHashMap<String, usize>,
    writes: index::PreparedReplacements,
    repeated_owners: FxHashSet<CallableId>,
    payloads: payloads::ReplacementPayloads,
    snapshot_owners: FxHashMap<CallableId, (bool, bool)>,
    mutations: FxHashMap<usize, (FxHashSet<String>, bool)>,
    unsupported: FxHashMap<String, (FxHashSet<String>, bool)>,
    #[cfg(feature = "test-instrumentation")]
    lookups: ReplacementLookupProbe,
    #[cfg(feature = "test-instrumentation")]
    inherited_frames: ReplacementLookupProbe,
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
        let module_writer_phases = execution_phases::ModuleWriterPhases::default();
        let mut flow = ValueFlowResolution {
            inputs,
            facts,
            resolver,
            indexes,
            incoming,
            supers: &pass.super_constructors,
            replacements: None,
            replacement_read: false,
            nullish_memo: None,
            foreign_acquisition: false,
            used_parameters: fx_set(),
            parameters_enabled: false,
            initializing: Vec::new(),
            cache: &pass.parameter_cache,
            visiting: fx_set(),
            calls_visiting: fx_set(),
            default_origins: None,
            acquisition_context: None,
            parameter_frame: None,
            invocation_read: None,
            invocation_start: None,
            executing_read: None,
            module_writer_phases: &module_writer_phases,
            writer_dependencies: Default::default(),
            module_snapshot: None,
            member_initializer: None,
            member_read: None,
        };
        for (path, file, receivers, repeated_owners, writes) in prepared {
            let snapshot_owners = replacement_snapshot_owners(
                &file,
                sites.get(path.as_path()).map_or(&[], Vec::as_slice),
            );
            let admission = calls::prepare(
                &file,
                &receivers,
                &writes,
                &snapshot_owners,
                &repeated_owners,
                sites.get(path.as_path()).map_or(&[], Vec::as_slice),
            );
            let mut payloads = payloads::ReplacementPayloads::prepare(
                &mut flow,
                path,
                &file,
                &receivers,
                &writes,
                &repeated_owners,
                &foreign,
            );
            payloads.block_regions(&writes, &admission);
            #[cfg(feature = "test-instrumentation")]
            let payloads = {
                let mut payloads = payloads;
                probe.hot_file_indexes += 1;
                let (writes, regions) = writes.retained_counts();
                probe.retained_write_indices += writes;
                probe.region_segments += regions;
                probe.append(std::mem::take(&mut payloads.probe));
                payloads
            };
            let (mutations, unsupported) = replacement_mutations(&file, &receivers, &writes);
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
                    #[cfg(feature = "test-instrumentation")]
                    inherited_frames: Default::default(),
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
}

include!("replacement_queries.rs");
include!("replacement_metadata.rs");

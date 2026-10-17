impl ReplacementCatalog {
    #[cfg(feature = "test-instrumentation")]
    pub(super) fn snapshot(&self) -> ReplacementProbeSnapshot {
        let mut probe = self.probe.clone();
        for file in self.files.values() {
            probe.lookup_count += file.lookups.count();
            probe.inherited_frame_qualifications += file.inherited_frames.count();
            probe.prefix_searches += file.writes.prefix_searches.count();
        }
        probe.rhs.sort_by(|left, right| {
            (&left.file, left.write_index).cmp(&(&right.file, right.write_index))
        });
        probe
    }
    pub(super) fn possible_repeat(
        &self,
        path: &std::path::Path,
        offset: u32,
        owner: Option<CallableId>,
    ) -> bool {
        owner.is_some()
            || self.file(path).is_some_and(|file| {
                file.writes
                    .region_at(owner, offset)
                    .is_some_and(|id| file.writes.facts.regions[id].may_repeat)
            })
    }
    pub(super) fn mutated_member(
        &self,
        source: &std::path::Path,
        target: &ResolvedCallTarget,
        member: &str,
    ) -> bool {
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
            .is_some_and(|writers| writers.foreign(member, source))
    }
    pub(super) fn file(&self, path: &std::path::Path) -> Option<&ReplacementFile> {
        self.files.get(path)
    }
}

impl ReplacementFile {
    pub(super) fn acquisition_repeats(&self, offset: u32, owner: Option<CallableId>) -> bool {
        self.writes
            .region_at(owner, offset)
            .is_some_and(|region| self.writes.facts.regions[region].may_repeat)
    }

    #[cfg(feature = "test-instrumentation")]
    pub(super) fn record_inherited_frames(&self) {
        self.inherited_frames.record(1);
    }
    pub(super) fn snapshot_owner(&self, owner: CallableId) -> Option<(bool, bool)> {
        self.snapshot_owners.get(&owner).copied()
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
        if owner.is_none() && region.is_some_and(|id| !self.writes.facts.regions[id].may_repeat) {
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

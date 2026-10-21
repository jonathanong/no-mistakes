use super::*;

impl ValueFlowResolution<'_, '_> {
    pub(in super::super) fn receiver_history_invalidated(
        &self,
        source: (&std::path::Path, usize, &str),
        projection: &[&str],
        reader: (receiver_reads::ReadId, &receiver_reads::ReceiverReads<'_>),
        possible: bool,
    ) -> bool {
        let (source, scope, name) = source;
        let (read, reads) = reader;
        self.indexes.file(self.facts, source).is_some_and(|index| {
            (if possible {
                index.values.possible_receiver_mutations.as_ref()
            } else {
                index.values.receiver_mutations.as_ref()
            })
            .and_then(|receivers| receivers.get(&scope))
            .and_then(|names| names.get(name))
            .is_some_and(|history| {
                history.assignment_invalidated_guarded_reads(
                    projection,
                    &|writer| {
                        reads.offset(read, source, writer).known().or_else(|| {
                            let (file, acquisition) = self.module_snapshot.as_ref()?;
                            let writer = writer?;
                            // This qualified proof precedes the writer's
                            // execution altogether; zero orders before
                            // every authored write in that callable.
                            (file == source
                                && self.writer_after_module_snapshot(source, writer, *acquisition))
                            .then_some(0)
                        })
                    },
                    &|| reads.default_owner(read, source),
                    &|guards| self.module_provider_summary(source, guards),
                )
            })
        })
    }
}

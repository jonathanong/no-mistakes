use super::*;

impl ImportCollector {
    /// Reuse the ordinary alias/root traversal, retaining its final exact paths
    /// in a separate sparse channel. These writes cannot erase a known target.
    pub(in super::super) fn remember_possible_escape(
        &mut self,
        key: &(usize, String),
        path: &[Option<String>],
        assigned: Option<(Option<String>, u32)>,
        tracking: &super::super::mutation_metadata::Tracking<'_>,
    ) {
        let Some((member, completed_at)) = assigned else {
            return;
        };
        let metadata = tracking.metadata;
        let histories = std::sync::Arc::make_mut(
            self.value_flow
                .possible_receiver_mutations
                .get_or_insert_with(|| std::sync::Arc::new(fx_map())),
        );
        let history = histories
            .entry(key.0)
            .or_default()
            .entry(key.1.clone())
            .or_default();
        std::sync::Arc::make_mut(history).record_assignment(
            path,
            member,
            super::super::mutation_assignments::AssignmentTiming {
                started_at: metadata.offset,
                context: metadata.context,
                completed: super::super::mutation_history::ValueAssignedMutation {
                    offset: completed_at,
                    unconditional_offset: None,
                    writer: super::super::mutation_history::ValueAssignedWriter::single(
                        metadata.context,
                    ),
                },
            },
            tracking.propagated,
            None,
            (None, None),
        );
    }
}

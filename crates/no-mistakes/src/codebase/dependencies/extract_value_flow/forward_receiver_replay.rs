use super::*;

impl ImportCollector {
    pub(super) fn replay_forward_write(
        &mut self,
        writer: &mut ForwardWrite,
        possible: Vec<mutation_projection::PendingMutation<nested::RootRead>>,
        aliases: &mut Option<mutation_alias_index::MutationAliasIndex>,
    ) {
        // Late aliases use their authored acquisition coordinates in both channels.
        writer.metadata.foreign_before = self.mutable_receiver_origin_watermark();
        for (sources, possible) in [
            (std::mem::take(&mut writer.sources), false),
            (possible, true),
        ] {
            if sources.is_empty() {
                continue;
            }
            #[cfg(feature = "test-instrumentation")]
            {
                self.value_flow.reference_forward_writer_replays += 1;
            }
            let info = MutationInfo {
                metadata: writer.metadata,
                assigned_member: writer.member.as_deref(),
                completed_at: writer.completed_at,
                has_assignment: writer.assigned,
                proof: Some(&writer.proof),
            };
            let mut affected = mutation_projection::Affected::default();
            let previous =
                std::mem::replace(&mut self.receiver_escapes.projecting_possible, possible);
            self.project_member_mutation(
                PendingQueue::from_vec(sources),
                None,
                info,
                &mut affected,
            );
            self.receiver_escapes.projecting_possible = previous;
            if !possible {
                let aliases = aliases.get_or_insert_with(|| self.prepare_mutation_alias_index());
                self.invalidate_mutation_bindings_indexed(
                    &affected,
                    info.metadata.context,
                    info.metadata.offset,
                    true,
                    Some(aliases),
                );
            }
        }
    }
}

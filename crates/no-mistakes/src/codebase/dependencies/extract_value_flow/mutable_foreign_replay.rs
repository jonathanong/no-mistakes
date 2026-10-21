use super::super::providers::ForeignProviderIndex;
use super::*;

impl ImportCollector {
    pub(super) fn replay_foreign(
        &mut self,
        index: &ForeignProviderIndex,
        references: Vec<nested::ReceiverWrite>,
        owner: CallableId,
        info: MutationInfo<'_>,
        aliases: &mut Option<mutation_alias_index::MutationAliasIndex>,
    ) {
        for (reference, provider) in &index.module_references {
            self.active_module_provider = Some(std::sync::Arc::clone(provider));
            let mut affected = mutation_projection::Affected::default();
            self.project_member_mutation(
                mutation_projection::PendingQueue::new((reference.clone(), false, true, true)),
                None,
                info,
                &mut affected,
            );
        }
        self.active_module_provider = None;
        if references.is_empty() {
            return;
        }
        let aliases = aliases.get_or_insert_with(|| self.prepare_mutation_alias_index());
        let mut affected = mutation_projection::Affected::default();
        self.project_member_mutation(
            mutation_projection::PendingQueue::from_vec(
                references
                    .into_iter()
                    .map(|reference| (reference, false, true, true))
                    .collect(),
            ),
            None,
            info,
            &mut affected,
        );
        self.invalidate_mutation_bindings_indexed(
            &affected,
            Some(owner),
            info.metadata.offset,
            true,
            Some(aliases),
        );
    }
    pub(super) fn replay_possible_foreign(
        &mut self,
        index: &ForeignProviderIndex,
        owner: Option<CallableId>,
        info: MutationInfo<'_>,
    ) {
        let mut references = index.references(owner, info.metadata.foreign_before);
        references.extend(
            index
                .module_references
                .iter()
                .map(|(reference, _)| reference.clone()),
        );
        if references.is_empty() {
            return;
        }
        let previous = std::mem::replace(&mut self.receiver_escapes.projecting_possible, true);
        let mut affected = mutation_projection::Affected::default();
        self.project_member_mutation(
            mutation_projection::PendingQueue::from_vec(
                references
                    .into_iter()
                    .map(|reference| (reference, false, true, true))
                    .collect(),
            ),
            None,
            info,
            &mut affected,
        );
        self.receiver_escapes.projecting_possible = previous;
    }
}

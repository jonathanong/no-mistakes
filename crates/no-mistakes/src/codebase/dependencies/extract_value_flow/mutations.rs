use super::*;

impl ImportCollector {
    pub(in super::super) fn invalidate_value_write(
        &mut self,
        scope: usize,
        name: &str,
        offset: u32,
    ) {
        self.invalidate_value_write_with_path(scope, name, offset, None, offset);
    }
    pub(in super::super) fn invalidate_value_write_with_path(
        &mut self,
        scope: usize,
        name: &str,
        offset: u32,
        receiver: Option<&[Option<String>]>,
        completed_at: u32,
    ) {
        let (base, member) = name
            .split_once('.')
            .map_or((name, false), |(base, _)| (base, true));
        if !member
            || self
                .receiver_conditional_region
                .is_some_and(|(_, admitted)| admitted != offset)
        {
            self.receiver_conditional_region = None;
        }
        if !member
            && (self.prior_member_mutations.requires_proof(scope, base)
                || self.has_mutable_receiver_binding(scope, base))
            && !self.prepared_mutable_root_write(scope, base)
        {
            self.fresh_receiver_effects.rebind(scope, base);
        }
        let mut affected = mutation_projection::Affected::default();
        affected.insert(scope, base);
        if member && self.collect_value_flow {
            let metadata = self.receiver_mutation_metadata(offset);
            self.record_phase_scope(scope);
            self.prepare_value_references();
            let mut parts = receiver.map_or_else(
                || {
                    name.split('.')
                        .skip(1)
                        .map(|member| Some(member.to_string()))
                        .collect::<Vec<_>>()
                },
                |path| path.to_vec(),
            );
            // A nested alias writes the same member on its original receiver.
            // Keep exact assignment metadata alongside every propagated path.
            let assigned_member = match receiver {
                Some(path) => path.last().and_then(Option::as_deref),
                None => name.rsplit_once('.').map(|(_, member)| member),
            };
            let assigned = parts.pop().map(|member| (member, completed_at));
            let has_assignment = assigned.is_some();
            let acquired_at = self.receiver_acquisition_offset(offset);
            let reads = vec![Some((acquired_at, self.current_function_id())); parts.len()];
            let info = mutation_projection::MutationInfo {
                metadata,
                assigned_member,
                completed_at,
                has_assignment,
                proof: None,
            };
            let root_read = (acquired_at, self.current_function_id());
            let receiver = ((scope, base.to_string()), parts);
            if self.receiver_only_bindings.is_none() {
                debug_assert!(self.deferred_mutable_writes.is_none());
                let pending = vec![((receiver, reads, ()), false, true, false)];
                self.project_member_mutation(pending, assigned, info, &mut affected);
            } else {
                let pending = vec![((receiver, reads, root_read), false, true, false)];
                self.project_member_mutation(pending, assigned, info, &mut affected);
            }
        }
        self.invalidate_mutation_bindings(&affected, self.current_function_id(), offset, member);
    }
}

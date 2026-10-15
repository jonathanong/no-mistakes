use super::*;
use identity_metadata::{map_between, FreshCapture};
use mutation_history::{ValueAssignedMutations, ValueAssignedWriter, ValueMutationHistory};

pub(super) fn reference_replaced(
    root: &ValueMutationHistory,
    path: &[Option<String>],
    reads: &[nested::ReceiverRead],
    region: Option<u32>,
    effects: Option<&fresh_receiver_effects::FreshReceiverEffects>,
    phases: Option<active_reference_phases::PhaseCursor<'_>>,
) -> bool {
    if path.len() != reads.len() {
        return false;
    }
    let Some((Some(member), prefix)) = path.split_last() else {
        return false;
    };
    let Some(Some((offset, context))) = reads.last() else {
        return false;
    };
    let mut node = root;
    let mut phase_node = phases;
    for segment in prefix {
        if let Some(effects) = effects {
            effects.note_identity_visit();
        }
        let Some(member) = segment.as_deref() else {
            return false;
        };
        let Some(child) = node.members.get(member) else {
            return false;
        };
        node = child;
        phase_node = phase_node.map(|cursor| cursor.child(member));
    }
    let Some(capture) = node
        .assigned
        .as_ref()
        .and_then(|assigned| assigned.definitely_replaced(member, *offset, *context, region))
    else {
        return false;
    };
    if phase_node.is_some_and(|cursor| {
        !cursor.allowed()
            || cursor.changed_after_capture(member, capture, reads.last().copied().flatten())
    }) {
        return false;
    }
    if capture.reads.len() != prefix.len()
        || effects.is_some_and(|effects| effects.epoch != capture.epoch)
    {
        return false;
    }
    if effects.is_some_and(|effects| {
        effects.called_between(capture.origin, reads.last().copied().flatten())
    }) {
        return false;
    }
    matching_ancestors(root, prefix, reads, capture, effects, phases)
}
fn matching_ancestors(
    mut node: &ValueMutationHistory,
    path: &[Option<String>],
    reads: &[nested::ReceiverRead],
    capture: &FreshCapture,
    effects: Option<&fresh_receiver_effects::FreshReceiverEffects>,
    phases: Option<active_reference_phases::PhaseCursor<'_>>,
) -> bool {
    let mut phase_node = phases;
    for ((member, current), captured) in path.iter().zip(reads).zip(capture.reads.iter()) {
        if let Some(effects) = effects {
            effects.note_identity_visit();
        }
        let Some(member) = member else {
            return false;
        };
        if current.is_none()
            || captured.is_none()
            || effects.is_some_and(|effects| effects.called_between(*captured, *current))
        {
            return false;
        }
        if current != captured
            && (phase_node.is_some_and(|cursor| cursor.changed(member, *current, *captured))
                || node.has_unassigned
                || node
                    .assigned
                    .as_ref()
                    .is_some_and(|assigned| assigned.identity_changed(member, *current, *captured)))
        {
            return false;
        }
        let Some(child) = node.members.get(member) else {
            return false;
        };
        node = child;
        phase_node = phase_node.map(|cursor| cursor.child(member));
    }
    true
}
impl ValueAssignedMutations {
    fn identity_changed(
        &self,
        member: &str,
        left: nested::ReceiverRead,
        right: nested::ReceiverRead,
    ) -> bool {
        let (Some((left_offset, context)), Some((right_offset, other))) = (left, right) else {
            return true;
        };
        if self
            .replacements
            .as_ref()
            .and_then(|events| events.identity.as_ref())
            .is_some_and(|identity| identity.unknown.changed(left, right))
        {
            return true;
        }
        let propagated = self.propagated.as_ref().map(|writes| &writes.aggregate);
        let writes = [
            self.members.get(member),
            self.dynamic.as_ref(),
            propagated.and_then(|writes| writes.members.get(member)),
            propagated.and_then(|writes| writes.dynamic.as_ref()),
        ];
        if writes
            .iter()
            .flatten()
            .any(|write| context != other || write.writer != ValueAssignedWriter::single(context))
        {
            return true;
        }
        if self
            .early
            .as_ref()
            .is_some_and(|early| early.identity_uncertain(member, left_offset, right_offset, self))
        {
            return true;
        }
        if writes.iter().all(Option::is_none) {
            return false;
        }
        let Some(events) = &self.replacements else {
            return true;
        };
        events
            .members
            .get(member)
            .is_some_and(|writes| map_between(writes, left_offset, right_offset))
            || map_between(&events.dynamic, left_offset, right_offset)
            || events
                .identity
                .as_ref()
                .is_some_and(|identity| identity.changed(member, left_offset, right_offset))
    }
}

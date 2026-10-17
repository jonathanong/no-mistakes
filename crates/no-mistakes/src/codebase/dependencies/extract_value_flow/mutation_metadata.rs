use super::*;

#[derive(Clone, Copy)]
pub(super) struct MutationMetadata {
    pub(super) offset: u32,
    pub(super) context: Option<CallableId>,
    pub(super) fresh: bool,
    pub(super) unconditional: bool,
    pub(super) region: Option<u32>,
    pub(super) epoch: u64,
    pub(super) phase: Option<(u32, u32)>,
    pub(super) foreign_before: usize,
}

impl ImportCollector {
    pub(super) fn receiver_mutation_metadata(&mut self, offset: u32) -> MutationMetadata {
        #[cfg(feature = "test-instrumentation")]
        {
            self.value_flow.reference_metadata_captures += 1;
        }
        MutationMetadata {
            offset,
            context: self.current_function_id(),
            fresh: self.receiver_replacement_is_fresh,
            unconditional: !self.receiver_write_is_conditional
                && self.receiver_replacement_is_fresh,
            region: self
                .receiver_conditional_region
                .filter(|(_, admitted)| *admitted == offset && self.receiver_replacement_is_fresh)
                .map(|(token, _)| token.get()),
            epoch: self.fresh_receiver_effects.epoch,
            phase: self
                .fresh_receiver_effects
                .phases
                .as_ref()
                .filter(|_| self.receiver_replacement_is_fresh)
                .and_then(|phases| phases.borrow().capture_phase(self.assignment_target_start)),
            foreign_before: self.mutable_receiver_origin_watermark(),
        }
    }
}

pub(super) struct Tracking<'a> {
    pub(super) metadata: &'a MutationMetadata,
    pub(super) propagated: bool,
    pub(super) root_read: Option<nested::RootRead>,
}

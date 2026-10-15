use super::*;

// File-local target starts join phase descriptors to normal-visitor ownership.
// Frames partition callable/catalog contexts without retaining a scope replay.
pub(in super::super) struct PhaseReceipts {
    owner: Option<CallableId>,
    pub(super) active: bool,
    pub(super) scopes: FxHashMap<u32, usize>,
}
pub(in super::super) struct ReceiptFrame {
    previous: Option<PhaseReceipts>,
}
impl ImportCollector {
    pub(in super::super) fn begin_phase_receipts(
        &mut self,
        target: &AssignmentTarget<'_>,
    ) -> Option<ReceiptFrame> {
        let owner = self.current_function_id();
        if !self.collect_value_flow
            || self.destructuring_phase_owner == Some(owner)
            || !matches!(
                target,
                AssignmentTarget::ArrayAssignmentTarget(_)
                    | AssignmentTarget::ObjectAssignmentTarget(_)
            )
        {
            return None;
        }
        Some(ReceiptFrame {
            previous: self.destructuring_phase_receipts.replace(PhaseReceipts {
                owner,
                active: true,
                scopes: fx_map(),
            }),
        })
    }
    pub(in super::super) fn end_phase_receipts(&mut self, frame: Option<ReceiptFrame>) {
        if let Some(frame) = frame {
            self.destructuring_phase_receipts = frame.previous;
        }
    }
    pub(super) fn record_phase_scope(&mut self, scope: usize) {
        let Some(target) = self.assignment_target_start else {
            return;
        };
        let owner = self.current_function_id();
        let Some(receipts) = self
            .destructuring_phase_receipts
            .as_mut()
            .filter(|receipts| receipts.active && receipts.owner == owner)
        else {
            return;
        };
        let previous = receipts.scopes.insert(target, scope);
        if let Some(phases) = &mut self.fresh_receiver_effects.phases {
            phases.resolve(target, scope, &self.prior_member_mutations);
        }
        #[cfg(feature = "test-instrumentation")]
        if previous.is_none() {
            self.value_flow.early_assignment_scope_receipts += 1;
        }
        #[cfg(not(feature = "test-instrumentation"))]
        let _ = previous;
    }
}

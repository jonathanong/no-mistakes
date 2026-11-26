use super::*;
use std::sync::Arc;

type RootPhaseEvent = ((usize, std::rc::Rc<str>), u32, ValueExpression, bool);

pub(in super::super) struct RootPhaseCapture {
    pub(super) catalog: Arc<phase_catalog::PhaseCatalog>,
    pub(super) first_changes: FxHashMap<std::rc::Rc<str>, u32>,
    pub(super) points: FxHashMap<u32, phase_walk::RootPhaseWrite>,
    pub(super) owner: Option<CallableId>,
    pub(super) conditional_depth: usize,
    pub(super) events: Vec<RootPhaseEvent>,
}
impl RootPhaseCapture {
    pub(super) fn first_changes(
        root_writes: &[phase_walk::RootPhaseWrite],
    ) -> FxHashMap<std::rc::Rc<str>, u32> {
        let mut first_changes: FxHashMap<std::rc::Rc<str>, u32> = fx_map();
        for point in root_writes {
            #[cfg(feature = "test-instrumentation")]
            root_target_metrics::phase_key(2);
            if let Some(phase) = first_changes.get_mut(point.name.as_ref()) {
                *phase = (*phase).min(point.phase);
            } else {
                #[cfg(feature = "test-instrumentation")]
                root_target_metrics::phase_key(3);
                first_changes.insert(
                    phase_changed_roots::shared_names::copy_name(&point.name),
                    point.phase,
                );
            }
        }
        first_changes
    }
}
impl ImportCollector {
    pub(super) fn prepared_mutable_root_write(&self, scope: usize, name: &str) -> bool {
        self.has_mutable_receiver_binding(scope, name)
            && self
                .receiver_root_phases
                .as_ref()
                .filter(|capture| capture.owner == self.current_function_id())
                .and_then(|capture| capture.points.get(&self.assignment_target_start?))
                .is_some_and(|point| point.name.as_ref() == name)
    }
    pub(super) fn mutable_root_unchanged(&self, name: &str, offset: u32) -> bool {
        self.receiver_root_phases
            .as_ref()
            .filter(|capture| capture.owner == self.current_function_id())
            .is_none_or(|capture| {
                capture.first_changes.get(name).is_none_or(|first| {
                    capture
                        .catalog
                        .phase(offset)
                        .is_some_and(|read| read < *first)
                })
            })
    }
    pub(super) fn capture_mutable_root_phase(
        &mut self,
        key: (usize, &str),
        target: u32,
        value: &ValueExpression,
        unconditional: bool,
    ) {
        let owner = self.current_function_id();
        let Some(capture) = &mut self.receiver_root_phases else {
            return;
        };
        if capture.owner != owner {
            return;
        }
        let point = capture
            .points
            .get(&target)
            .filter(|point| point.name.as_ref() == key.1)
            .map(|point| {
                (
                    point.phase,
                    point.guaranteed,
                    phase_changed_roots::shared_names::copy_name(&point.name),
                )
            });
        let Some((phase, local_guaranteed, name)) = point else {
            return;
        };
        let guaranteed = unconditional
            && local_guaranteed
            && self.receiver_write_conditional_depth == capture.conditional_depth
            && self.repeat_depth == self.call_repeat_base;
        capture
            .events
            .push(((key.0, name), phase, value.clone(), guaranteed));
    }
}

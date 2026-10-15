use super::*;

impl ValueFlowResolution<'_, '_> {
    pub(super) fn invocation_argument(
        &mut self,
        caller: &std::path::Path,
        call: &ValueCall,
        mut index: usize,
    ) -> Option<FlowValue> {
        let origins = self.default_origins.take();
        let mut callee = self.value(caller, &call.callee);
        self.default_origins = origins;
        loop {
            callee = match callee {
                FlowValue::Annotated { executor, .. }
                | FlowValue::Returned { executor, .. }
                | FlowValue::Provenance { executor, .. } => *executor,
                other => {
                    callee = other;
                    break;
                }
            };
        }
        if let FlowValue::Bound { arguments, .. } = callee {
            for bound in arguments {
                if bound.spread_at.is_some_and(|spread| index >= spread) {
                    return Some(FlowValue::Unknown);
                }
                if let Some(argument) = bound.arguments.get(index) {
                    return Some(self.value(&bound.file, argument));
                }
                index -= bound.arguments.len();
            }
        }
        if call.spread_at.is_some_and(|spread| index >= spread) {
            return Some(FlowValue::Unknown);
        }
        call.arguments
            .get(index)
            .map(|argument| self.value(caller, argument))
    }
}

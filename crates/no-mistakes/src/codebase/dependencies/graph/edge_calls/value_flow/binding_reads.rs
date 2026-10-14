use super::*;

impl ValueFlowResolution<'_, '_> {
    pub(super) fn expression_read(
        &self,
        path: &std::path::Path,
        offset: u32,
        context: Option<CallableId>,
    ) -> (u32, Option<CallableId>) {
        let Some((source, invocation, caller, owner)) = &self.invocation_read else {
            return (offset, context);
        };
        if source != path {
            return (offset, context);
        }
        self.executing_read
            .filter(|(owner, _, _)| Some(*owner) == context)
            .map(|(_, offset, context)| (offset, context))
            .or_else(|| (context == Some(*owner)).then_some((*invocation, *caller)))
            .unwrap_or((offset, context))
    }

    /// Captured module bindings must exist and remain unchanged at execution.
    /// Local TDZ checks and stored initializer acquisition keep source offsets.
    pub(super) fn binding_read(
        &self,
        path: &std::path::Path,
        scope: usize,
        source_offset: u32,
        source_context: Option<CallableId>,
        offset: u32,
        context: Option<CallableId>,
    ) -> (u32, u32, Option<CallableId>) {
        let Some((source, invocation, caller, _)) = &self.invocation_read else {
            return (source_offset, offset, context);
        };
        if scope != 0 || source != path || source_context.is_none() {
            return (source_offset, offset, context);
        }
        if let Some((_, read_at, read_context)) = self
            .executing_read
            .filter(|(owner, _, _)| Some(*owner) == source_context)
        {
            return (read_at, read_at, read_context);
        }
        self.acquired_binding_read(path, source_context.expect("captured binding"))
            .unwrap_or((*invocation, *invocation, *caller))
    }

    /// Lazy returned fields borrow their existing owner-qualified acquisition
    /// frames. All callers must see initialization and precede later mutation.
    fn acquired_binding_read(
        &self,
        path: &std::path::Path,
        owner: CallableId,
    ) -> Option<(u32, u32, Option<CallableId>)> {
        let root = self.acquisition_context.as_ref()?;
        let mut available_at = u32::MAX;
        let mut read_at = 0;
        let mut context = None;
        for frame in receiver_owned_contexts::frames(root, path, owner) {
            let Some(OriginNode {
                kind: OriginKind::ReceiverUse(_, read),
                ..
            }) = frame
            else {
                return Some((0, u32::MAX, None));
            };
            if read.file != path || context.is_some_and(|prior| prior != read.context) {
                return Some((0, u32::MAX, None));
            }
            available_at = available_at.min(read.offset);
            read_at = read_at.max(read.offset);
            context = Some(read.context);
        }
        context.map(|context| (available_at, read_at, context))
    }

    /// Only the synchronous returned body uses this scalar phase. The inline
    /// invocation identity and parameter agreement cache key remain unchanged.
    pub(super) fn enter_returned_read(
        &mut self,
        caller: &std::path::Path,
        call: &ValueCall,
        file: &std::path::Path,
        owner: CallableId,
    ) {
        let Some((source, offset, context, outer_owner)) = &self.invocation_read else {
            return;
        };
        if source != caller || source != file {
            self.executing_read = None;
            return;
        }
        let inherited = self
            .executing_read
            .filter(|(executing, _, _)| Some(*executing) == call.context)
            .map(|(_, offset, context)| (offset, context))
            .or_else(|| (call.context == Some(*outer_owner)).then_some((*offset, *context)));
        let (read_at, read_context) =
            inherited.unwrap_or((call.end.saturating_sub(1), call.context));
        self.executing_read = Some((owner, read_at, read_context));
    }
}

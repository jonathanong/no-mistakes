use super::*;

impl ValueFlowResolution<'_, '_> {
    pub(super) fn enter_inline_read(
        &mut self,
        caller: &std::path::Path,
        call: &ValueCall,
        file: &std::path::Path,
        owner: CallableId,
    ) {
        if super::inline_inputs::inline_callee(&call.callee) {
            self.invocation_read = Some(default_origins::InvocationRead {
                file: caller.to_path_buf(),
                offset: call.end.saturating_sub(1),
                context: call.context,
                owner: (file.to_path_buf(), owner),
            });
        }
    }
    pub(super) fn expression_read(
        &self,
        path: &std::path::Path,
        offset: u32,
        context: Option<CallableId>,
    ) -> (u32, Option<CallableId>) {
        let Some(read) = &self.invocation_read else {
            return (offset, context);
        };
        if read.file != path {
            return (offset, context);
        }
        self.executing_read
            .filter(|(owner, _, _)| Some(*owner) == context)
            .map(|(_, offset, context)| (offset, context))
            .or_else(|| {
                (read.owner.0 == path && context == Some(read.owner.1))
                    .then_some((read.offset, read.context))
            })
            .unwrap_or((offset, context))
    }

    /// Captured bindings must exist and remain unchanged at execution.
    /// Local TDZ checks and stored initializer acquisition keep source offsets.
    pub(super) fn binding_read(
        &self,
        path: &std::path::Path,
        declared_owner: Option<CallableId>,
        source_offset: u32,
        source_context: Option<CallableId>,
        offset: u32,
        context: Option<CallableId>,
    ) -> (u32, u32, Option<CallableId>) {
        let Some(read) = &self.invocation_read else {
            return (source_offset, offset, context);
        };
        if read.file != path || source_context.is_none() || declared_owner == source_context {
            return (source_offset, offset, context);
        }
        if let Some((_, read_at, read_context)) = self
            .executing_read
            .filter(|(owner, _, _)| Some(*owner) == source_context)
        {
            return (read_at, read_at, read_context);
        }
        self.acquired_binding_read(path, source_context.expect("captured binding"))
            .unwrap_or((read.offset, read.offset, read.context))
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
        let Some(read) = &self.invocation_read else {
            return;
        };
        if read.file != caller || read.file != file {
            self.executing_read = None;
            return;
        }
        let inherited = self
            .executing_read
            .filter(|(executing, _, _)| Some(*executing) == call.context)
            .map(|(_, offset, context)| (offset, context))
            .or_else(|| {
                (read.owner.0 == caller && call.context == Some(read.owner.1))
                    .then_some((read.offset, read.context))
            });
        let (read_at, read_context) =
            inherited.unwrap_or((call.end.saturating_sub(1), call.context));
        self.executing_read = Some((owner, read_at, read_context));
    }
}

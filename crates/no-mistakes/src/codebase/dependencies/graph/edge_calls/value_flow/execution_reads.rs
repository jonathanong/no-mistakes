use super::*;
use execution_quality::ExecutionPhase;

impl<'a> ValueFlowResolution<'a, '_> {
    fn set_execution_phase(
        &mut self,
        file: &std::path::Path,
        owner: CallableId,
        phase: ExecutionPhase,
    ) {
        self.executing_read = self
            .inputs
            .graph_files
            .visible_path(file)
            .filter(|_| !matches!(phase, ExecutionPhase::Absent))
            .map(|file| (file, owner, phase));
    }
    pub(super) fn enter_inline_read(
        &mut self,
        caller: &std::path::Path,
        call: &ValueCall,
        file: &std::path::Path,
        owner: CallableId,
    ) {
        if super::inline_inputs::inline_callee(&call.callee) {
            self.invocation_start = Some(call.offset);
            self.invocation_read = Some(default_origins::InvocationRead {
                file: caller.to_path_buf(),
                offset: call.end.saturating_sub(1),
                context: call.context,
                owner: (file.to_path_buf(), owner),
                once: call.once,
                repeated: call.repeated,
            });
        }
    }
    pub(super) fn expression_phase(
        &self,
        path: &std::path::Path,
        offset: u32,
        context: Option<CallableId>,
    ) -> ExecutionPhase {
        if let Some((_, _, phase)) = self
            .executing_read
            .filter(|(file, owner, _)| *file == path && Some(*owner) == context)
        {
            return phase;
        }
        // Arguments execute in the caller, while the derived parameter phase is
        // owned by the inline callee. Preserve the argument's authored position,
        // but carry repetition from that same invocation's canonical callers.
        // A transitive binding initializer outside this call acquires separately.
        let repeated = self.executing_read.is_some_and(|(file, owner, phase)| {
            file == path
                && phase.is_repeated()
                && self.invocation_read.as_ref().is_some_and(|read| {
                    read.file == path
                        && read.owner.0 == path
                        && read.owner.1 == owner
                        && read.context == context
                        && self.invocation_start.is_some_and(|start| start <= offset)
                        && offset <= read.offset
                })
        });
        let source = ExecutionPhase::Exact((offset, context)).repeated(repeated);
        self.invocation_read
            .as_ref()
            .filter(|read| {
                read.file == path && read.owner.0 == path && context == Some(read.owner.1)
            })
            .map(|read| ExecutionPhase::Exact((read.offset, read.context)).repeated(read.repeated))
            .unwrap_or(source)
    }
    pub(super) fn expression_read(
        &self,
        path: &std::path::Path,
        offset: u32,
        context: Option<CallableId>,
    ) -> (u32, Option<CallableId>, bool) {
        let phase = self.expression_phase(path, offset, context);
        let (offset, context) = phase.location().unwrap_or((offset, context));
        (offset, context, phase.is_repeated())
    }

    /// Each cached agreement derives its own phase from canonical incoming calls.
    pub(super) fn enter_parameter_read(
        &mut self,
        caller: &std::path::Path,
        call: &ValueCall,
        file: &std::path::Path,
        owner: CallableId,
    ) {
        if let Some(context) = call.context.filter(|context| {
            self.invocation_read
                .as_ref()
                .is_some_and(|read| read.owner.0 != caller || read.owner.1 != *context)
        }) {
            let phase = if caller == file {
                self.unique_execution_phase(file, context)
            } else {
                ExecutionPhase::Absent
            };
            let fallback = self
                .invocation_read
                .as_ref()
                .filter(|read| read.file == caller && read.file == file)
                .map_or(ExecutionPhase::Absent, |read| {
                    ExecutionPhase::Exact((read.offset, read.context)).repeated(read.repeated)
                });
            let repeated = call.repeated
                || self
                    .invocation_read
                    .as_ref()
                    .is_some_and(|read| read.repeated);
            self.set_execution_phase(file, owner, phase.or_absent(fallback).repeated(repeated));
            return;
        }
        self.enter_returned_read(caller, call, file, owner);
    }

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
        let repeated = call.repeated || read.repeated;
        if read.file != caller || read.file != file {
            self.set_execution_phase(file, owner, ExecutionPhase::Absent.repeated(repeated));
            return;
        }
        let inherited = self
            .executing_read
            .filter(|(file, executing, _)| *file == caller && Some(*executing) == call.context)
            .map(|(_, _, phase)| phase)
            .or_else(|| {
                (read.owner.0 == caller && call.context == Some(read.owner.1)).then_some(
                    ExecutionPhase::Exact((read.offset, read.context)).repeated(read.repeated),
                )
            });
        let phase = inherited
            .unwrap_or(ExecutionPhase::Exact((
                call.end.saturating_sub(1),
                call.context,
            )))
            .repeated(repeated);
        self.set_execution_phase(file, owner, phase);
    }
}

use super::*;
use execution_quality::ExecutionPhase;

enum Step {
    Owner(CallableId),
    Phase(u32, Option<CallableId>),
}

impl ValueFlowResolution<'_, '_> {
    /// Follow a closed chain of unique canonical callers once per parameter
    /// agreement. Floyd's walk rejects cycles without retaining a path or cap.
    pub(super) fn unique_execution_phase(
        &mut self,
        path: &std::path::Path,
        owner: CallableId,
    ) -> ExecutionPhase {
        let Some(read) = self.invocation_read.as_ref() else {
            return ExecutionPhase::Absent;
        };
        let mut repeated = read.repeated;
        if read.file != path || read.owner.0 != path {
            return ExecutionPhase::Absent.repeated(repeated);
        }
        let mut key = (path.to_path_buf(), owner);
        let (mut slow, mut fast) = (owner, owner);
        loop {
            key.1 = slow;
            slow = match self.execution_step(&key, &mut repeated) {
                Some(Step::Owner(owner)) => owner,
                Some(Step::Phase(offset, context)) => {
                    return ExecutionPhase::Exact((offset, context)).repeated(repeated);
                }
                None => return ExecutionPhase::Absent.repeated(repeated),
            };
            for _ in 0..2 {
                key.1 = fast;
                fast = match self.execution_step(&key, &mut repeated) {
                    Some(Step::Owner(owner)) => owner,
                    Some(Step::Phase(offset, context)) => {
                        return ExecutionPhase::Exact((offset, context)).repeated(repeated);
                    }
                    None => return ExecutionPhase::Absent.repeated(repeated),
                };
            }
            if slow == fast {
                return ExecutionPhase::Absent.repeated(repeated);
            }
        }
    }

    fn execution_step(
        &mut self,
        key: &(std::path::PathBuf, CallableId),
        repeated: &mut bool,
    ) -> Option<Step> {
        #[cfg(feature = "test-instrumentation")]
        self.cache.record_execution_step();
        if !self.used_parameters.contains(key) {
            self.used_parameters.insert(key.clone());
        }
        let index = self.indexes.file(self.facts, &key.0)?;
        let scope = index.values.scopes.get(&key.1)?;
        if index.exported_scopes.contains(scope) || !self.synchronous_callable(&key.0, key.1) {
            return None;
        }
        let [input] = self.incoming.get(key)?.as_slice() else {
            return None;
        };
        if input.callback || input.file != key.0 {
            return None;
        }
        let call = index
            .values
            .sites
            .get(&input.offset)
            .and_then(|sites| sites.get(&input.callee))
            .and_then(|key| index.values.calls.get(key))?;
        *repeated |= call.repeated;
        if call.bound || call.spread_at.is_some() {
            return None;
        }
        match call.context {
            None => Some(Step::Phase(call.end.saturating_sub(1), None)),
            Some(owner) => {
                let read = self.invocation_read.as_ref()?;
                if read.owner.0 == key.0 && read.owner.1 == owner {
                    Some(Step::Phase(read.offset, read.context))
                } else {
                    Some(Step::Owner(owner))
                }
            }
        }
    }
}

impl ValueFlowResolution<'_, '_> {
    /// A closed synchronous caller chain can prove that every execution starts
    /// after an independently acquired module snapshot, even when it repeats.
    /// Floyd's walk rejects cycles without retaining an extra caller graph.
    pub(super) fn writer_after_module_snapshot(
        &self,
        path: &std::path::Path,
        writer: CallableId,
        acquisition: u32,
    ) -> bool {
        let mut key = (path.to_path_buf(), writer);
        let (mut slow, mut fast) = (writer, writer);
        loop {
            key.1 = slow;
            slow = match self.module_writer_step(&key) {
                Some(Step::Owner(owner)) => owner,
                Some(Step::Phase(offset, _)) => return offset > acquisition,
                None => return false,
            };
            for _ in 0..2 {
                key.1 = fast;
                fast = match self.module_writer_step(&key) {
                    Some(Step::Owner(owner)) => owner,
                    Some(Step::Phase(offset, _)) => return offset > acquisition,
                    None => return false,
                };
            }
            if slow == fast { return false; }
        }
    }

    fn module_writer_step(&self, key: &(std::path::PathBuf, CallableId)) -> Option<Step> {
        let index = self.indexes.file(self.facts, &key.0)?;
        let scope = index.values.scopes.get(&key.1)?;
        if index.exported_scopes.contains(scope) || !self.synchronous_callable(&key.0, key.1) {
            return None;
        }
        let [input] = self.incoming.get(key)?.as_slice() else { return None; };
        if input.callback || input.file != key.0 { return None; }
        let call = index.values.sites.get(&input.offset)
            .and_then(|sites| sites.get(&input.callee))
            .and_then(|key| index.values.calls.get(key))?;
        if call.bound || call.spread_at.is_some() { return None; }
        Some(match call.context {
            Some(owner) => Step::Owner(owner),
            None => Step::Phase(input.offset, None),
        })
    }
}

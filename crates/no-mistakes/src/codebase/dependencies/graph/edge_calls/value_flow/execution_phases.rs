use super::*;

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
    ) -> Option<(u32, Option<CallableId>)> {
        let read = self.invocation_read.as_ref()?;
        if read.file != path || read.owner.0 != path {
            return None;
        }
        let mut key = (path.to_path_buf(), owner);
        let (mut slow, mut fast) = (owner, owner);
        loop {
            key.1 = slow;
            slow = match self.execution_step(&key)? {
                Step::Owner(owner) => owner,
                Step::Phase(offset, context) => return Some((offset, context)),
            };
            for _ in 0..2 {
                key.1 = fast;
                fast = match self.execution_step(&key)? {
                    Step::Owner(owner) => owner,
                    Step::Phase(offset, context) => return Some((offset, context)),
                };
            }
            if slow == fast {
                return None;
            }
        }
    }

    fn execution_step(&mut self, key: &(std::path::PathBuf, CallableId)) -> Option<Step> {
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

use super::*;
use execution_quality::ExecutionPhase;

impl ValueFlowResolution<'_, '_> {
    pub(super) fn parameter_acquisition_repeats(
        &mut self,
        caller: &std::path::Path,
        call: &ValueCall,
        path: &std::path::Path,
        owner: CallableId,
        unique: bool,
    ) -> bool {
        let Some(catalog) = self.replacements else {
            return false;
        };
        if !catalog.possible_repeat(caller, call.offset, call.context) {
            return false;
        }
        if caller != path {
            return true;
        }
        let Some(index) = self.indexes.file(self.facts, path) else {
            return true;
        };
        if !unique
            || call.bound
            || call.spread_at.is_some()
            || !self.synchronous_callable(path, owner)
            || index
                .values
                .scopes
                .get(&owner)
                .is_none_or(|scope| index.exported_scopes.contains(scope))
        {
            return true;
        }
        // Reuse the phase just derived for this incoming agreement. An explicit
        // repeated/absent phase must never fall back to another lexical proof.
        let qualified = self
            .executing_read
            .filter(|(file, executing, _)| *file == path && *executing == owner)
            .map(|(_, _, phase)| phase)
            .or_else(|| {
                self.invocation_read
                    .as_ref()
                    .filter(|read| {
                        read.file == path && read.owner.0 == path && read.owner.1 == owner
                    })
                    .map(|read| {
                        ExecutionPhase::Exact((read.offset, read.context)).repeated(read.repeated)
                    })
            });
        let phase = qualified
            .unwrap_or_else(|| self.source_execution_phase(path, owner))
            .repeated(call.repeated);
        // A repeated same-file default still acquires its method before the
        // body. Delegate to main's snapshot guards without changing its phase.
        !matches!(
            phase,
            ExecutionPhase::Exact((_, None)) | ExecutionPhase::Repeated(Some((_, None)))
        )
    }

}

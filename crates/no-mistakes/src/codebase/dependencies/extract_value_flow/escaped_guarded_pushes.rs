use super::*;

impl ImportCollector {
    /// Update common slots once per indexed push. Unequal or uncertain lengths
    /// never establish a new common slot; the guaranteed prefix stays valid.
    pub(in super::super) fn note_guarded_array_push(
        &mut self,
        scope: usize,
        name: &str,
        call: &ValueCall,
        ordinary: bool,
    ) -> Option<bool> {
        let mut summary = self
            .receiver_escapes
            .guarded_summaries
            .get_mut(&scope)
            .and_then(|names| {
                // RHS calls still operate on the previous array, not its replacement.
                if names.get(name)?.ready_at >= call.offset {
                    return None;
                }
                names.remove(name)
            })?;
        let mut common = summary
            .common
            .take()
            .unwrap_or_else(|| self.array_identities(&summary.seed));
        let certain = ordinary
            && !call.repeated
            && !self.conditional_execution
            && !self.receiver_write_is_conditional
            && call.context == summary.context
            && call.spread_at.is_none();
        if certain {
            if let Some(length) = summary.length {
                for (index, value) in call.arguments.iter().enumerate() {
                    if let Some(identity) = self.array_identity(value) {
                        common.insert((length + index).to_string(), identity);
                    }
                }
                summary.length = Some(length + call.arguments.len());
            }
        } else if !call.arguments.is_empty() {
            summary.length = None;
        }
        summary.common = Some(common);
        self.receiver_escapes
            .guarded_summaries
            .entry(scope)
            .or_default()
            .insert(name.to_owned(), summary);
        Some(certain)
    }
}

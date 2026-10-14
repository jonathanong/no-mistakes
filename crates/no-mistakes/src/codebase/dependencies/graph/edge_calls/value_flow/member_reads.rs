use super::*;

impl ValueFlowResolution<'_, '_> {
    pub(super) fn member_at(
        &mut self,
        base: FlowValue,
        member: &str,
        file: &std::path::Path,
        location: Option<(u32, Option<CallableId>)>,
    ) -> FlowValue {
        let context = location.and_then(|(_, context)| context);
        // A method read while executing a foreign callable cannot inherit the
        // caller's repeated tail. Module/copy initializers have no callable
        // source context and retain their acquisition snapshot.
        if self.foreign_acquisition && context.is_some() {
            if let FlowValue::Target(target) = base.executor() {
                if self
                    .replacements
                    .is_some_and(|catalog| catalog.mutated_member(target, member))
                {
                    self.replacement_read = true;
                    return FlowValue::Unknown;
                }
            }
        }
        let enabled = location.is_some()
            && self.indexes.file(self.facts, file).is_some_and(|index| {
                index
                    .values
                    .receiver_mutations
                    .as_ref()
                    .is_some_and(|receivers| {
                        context.map_or(receivers.module_writer, |context| {
                            receivers.writer_contexts.contains(&context)
                        })
                    })
            });
        let previous = self.member_read.take();
        if enabled {
            let (offset, context) = location.expect("enabled reads have a source location");
            self.member_read = Some((file.to_path_buf(), offset, context));
            if self
                .default_origins
                .as_ref()
                .is_some_and(|origins| origins.has_receivers || origins.has_receiver_reads)
            {
                self.record_value_receiver_read(file, offset, context);
            }
        }
        let value = self.member(base, member);
        self.member_read = previous;
        value
    }

    pub(super) fn member_read_location(
        expression: &ValueExpression,
    ) -> Option<(u32, Option<CallableId>)> {
        match expression {
            ValueExpression::Named {
                offset, context, ..
            } => Some((*offset, *context)),
            ValueExpression::Member(base, _) | ValueExpression::Awaited(base) => {
                Self::member_read_location(base)
            }
            _ => None,
        }
    }
}

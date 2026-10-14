use super::*;

impl ValueFlowResolution<'_, '_> {
    pub(super) fn member_at(
        &mut self,
        base: FlowValue,
        member: &str,
        file: &std::path::Path,
        offset: u32,
        context: Option<CallableId>,
    ) -> FlowValue {
        let enabled = self.indexes.file(self.facts, file).is_some_and(|index| {
            index.values.receiver_mutations.as_ref().is_some_and(|receivers| {
                context.map_or(receivers.module_writer, |context| {
                    receivers.writer_contexts.contains(&context)
                })
            })
        });
        let previous = self.member_read.take();
        if enabled {
            self.member_read = Some((file.to_path_buf(), offset, context));
            if self
                .default_origins
                .as_ref()
                .is_some_and(|origins| origins.has_receivers || origins.has_receiver_reads)
            {
                self.record_parameter_receiver_read(file, offset, context);
            }
        }
        let value = self.member(base, member);
        self.member_read = previous;
        value
    }

    pub(super) fn member_read_location(expression: &ValueExpression) -> (u32, Option<CallableId>) {
        match expression {
            ValueExpression::Named {
                offset, context, ..
            } => (*offset, *context),
            ValueExpression::Member(base, _) | ValueExpression::Awaited(base) => {
                Self::member_read_location(base)
            }
            _ => (0, None),
        }
    }
}

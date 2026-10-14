use super::*;

impl ValueFlowResolution<'_, '_> {
    pub(super) fn refresh_alias_receiver(
        &mut self,
        path: &std::path::Path,
        offset: u32,
        context: Option<CallableId>,
        hint: Option<&member_projection::MemberProjection<'_>>,
        value: &FlowValue,
    ) {
        member_projection::preserve_whole_receiver(hint, value, self.default_origins.as_deref());
        if hint.is_some_and(|hint| hint.whole_receiver.get())
            && context.is_some_and(|context| {
                self.indexes.file(self.facts, path).is_some_and(|index| {
                    index
                        .values
                        .receiver_mutations
                        .as_ref()
                        .is_some_and(|receivers| receivers.writer_contexts.contains(&context))
                })
            })
        {
            self.record_parameter_receiver_read(path, offset, context);
        }
    }
    pub(super) fn record_parameter_receiver_read(
        &mut self,
        file: &std::path::Path,
        offset: u32,
        context: Option<CallableId>,
    ) {
        let Some(origins) = self.default_origins.take() else {
            return;
        };
        self.default_origins = Some(if origins.has_receivers || origins.has_receiver_reads {
            self.cache.origin_node(
                OriginKind::ReceiverUse(
                    Some(origins),
                    default_origins::ReceiverRead {
                        file: file.to_path_buf(),
                        offset,
                        context,
                        invocation: true,
                        owned_context: None,
                    },
                ),
                1,
            )
        } else {
            origins
        });
    }
}

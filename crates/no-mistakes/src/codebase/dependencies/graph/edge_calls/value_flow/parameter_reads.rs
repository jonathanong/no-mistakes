use super::*;

impl ValueFlowResolution<'_, '_> {
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

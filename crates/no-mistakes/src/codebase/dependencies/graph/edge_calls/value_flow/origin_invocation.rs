use super::*;

impl ValueFlowResolution<'_, '_> {
    pub(in super::super) fn invocation_receiver_invalidated(
        &self,
        origin: &OriginKind,
        query: (&std::path::Path, Option<CallableId>),
        receivers_only: bool,
        receivers: bool,
        reader: (receiver_reads::ReadId, &receiver_reads::ReceiverReads<'_>),
        projection: &[&str],
    ) -> bool {
        let OriginKind::InvocationReceiver {
            read,
            history_file,
            history,
        } = origin
        else {
            return false;
        };
        if self.invocation_read.as_ref().is_some_and(|active| {
            !read.repeated
                && !active.repeated
                && active.owner == read.owner
                && active.owner.0 == query.0
                && query.1 == Some(active.owner.1)
        }) {
            return false;
        }
        // Supplier offsets only order their own history. Reader ownership is
        // file-qualified, while member writes follow the receiver projection.
        read.file != *history_file
            || if receivers_only {
                receivers
                    && history.assignment_invalidated_guarded_reads(
                        projection,
                        &|writer| {
                            if read.repeated {
                                None
                            } else if read.once
                                && read.owner.0 == *history_file
                                && writer == Some(read.owner.1)
                            {
                                reader.1.offset(reader.0, history_file, writer).known()
                            } else {
                                (writer == read.context).then_some(read.offset)
                            }
                        },
                        &|| None,
                        &|guards| self.module_provider_summary(history_file, guards),
                    )
            } else {
                read.owner.0 != query.0 || query.1 != Some(read.owner.1)
            }
    }
}

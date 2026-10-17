impl ValueFlowResolution<'_, '_> {
    /// Resolve acquisition at its source position, before receiver-origin
    /// invalidation turns the live receiver into an unknown method value.
    fn replacement_coordinates(
        &self,
        path: &std::path::Path,
        offset: u32,
        context: Option<CallableId>,
    ) -> Option<(u32, Option<CallableId>)> {
        if self.invocation_read.as_ref().is_some_and(|read| {
            read.file == path
                && ((read.owner.0 == path && context == Some(read.owner.1))
                    || self
                        .executing_read
                        .is_some_and(|(file, owner, _)| file == path && context == Some(owner)))
        }) {
            // A repeated phase without a proven location must delegate to the
            // ordinary repeated-read guard, never fabricate lexical timing.
            return self.expression_phase(path, offset, context).location();
        }
        if let (Some(owner), Some(catalog), Some(root)) = (
            context,
            self.replacements,
            self.acquisition_context.as_ref(),
        ) {
            if let Some(file) = catalog.file(path) {
                let Some((nonwriter, closed)) = file.snapshot_owner(owner) else {
                    return Some((offset, context));
                };
                let direct = [Some(root.as_ref())];
                let inherited;
                let bounded;
                let frames = if matches!(&root.kind, OriginKind::ReceiverUse(_, read)
                    if read.owns(path, context))
                {
                    direct.as_slice()
                } else {
                    bounded = receiver_owned_contexts::bounded_frames(root, path, owner);
                    if let Some(frames) = bounded.as_ref() {
                        frames.as_slice()
                    } else {
                        #[cfg(feature = "test-instrumentation")]
                        file.record_inherited_frames();
                        inherited = receiver_owned_contexts::frames(root, path, owner);
                        inherited.as_slice()
                    }
                };
                // Select every relevant owner-qualified acquisition branch.
                // Delegation adds no target: main still validates source reads,
                // including nested helper calls and writes after a method copy.
                let once = !self.foreign_acquisition && !frames.is_empty() && frames.iter().all(|frame| {
                    matches!(frame.map(|frame| &frame.kind), Some(OriginKind::ReceiverUse(_, read))
                        if read.file == path && read.owns(path, context) && read.context.is_none()
                            && !read.repeated && !catalog.possible_repeat(path, read.offset, read.context))
                });
                if once {
                    return None;
                }
                // Only a pure closed getter may adopt a repeated caller phase.
                if nonwriter && closed && frames.len() == 1 {
                    if let Some(OriginKind::ReceiverUse(_, read)) =
                        frames[0].map(|frame| &frame.kind)
                    {
                        if read.file == path && read.owns(path, context) {
                            return Some((read.offset, read.context));
                        }
                    }
                }
            }
        }
        Some((offset, context))
    }
}

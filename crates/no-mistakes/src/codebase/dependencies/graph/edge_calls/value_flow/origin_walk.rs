use super::*;

impl ValueFlowResolution<'_, '_> {
    pub(super) fn origins_invalidated(
        &self,
        file: &std::path::Path,
        offset: u32,
        contexts: (Option<CallableId>, Option<CallableId>),
        receivers_only: bool,
        hint: Option<&member_projection::MemberProjection<'_>>,
    ) -> bool {
        self.origins_invalidated_in_channel(file, offset, contexts, receivers_only, hint, false)
    }

    pub(super) fn origins_invalidated_in_channel(
        &self,
        file: &std::path::Path,
        offset: u32,
        contexts: (Option<CallableId>, Option<CallableId>),
        receivers_only: bool,
        hint: Option<&member_projection::MemberProjection<'_>>,
        possible: bool,
    ) -> bool {
        let (context, source_context) = contexts;
        let Some(root) = &self.default_origins else {
            return false;
        };
        enum Step<'a> {
            Visit(&'a OriginNode, receiver_reads::ReadId, bool, bool),
            Pop,
        }
        let mut pending = vec![Step::Visit(root, None, true, true)];
        let mut projection =
            hint.map_or_else(Vec::new, member_projection::MemberProjection::members);
        let hint_len = projection.len();
        let mut reads = receiver_reads::ReceiverReads::default();
        let mut visited = fx_set();
        while let Some(step) = pending.pop() {
            let Step::Visit(node, read, receivers, acquiring) = step else {
                projection.pop();
                continue;
            };
            let relevant = if receivers_only {
                node.has_receivers
            } else {
                node.has_defaults
            };
            if !relevant {
                continue;
            }
            if !visited.insert((
                node as *const OriginNode as usize,
                projection.clone(),
                read,
                receivers,
                acquiring,
            )) {
                continue;
            }
            match &node.kind {
                OriginKind::Union(left, right) => {
                    pending.push(Step::Visit(right, read, receivers, acquiring));
                    pending.push(Step::Visit(left, read, receivers, acquiring));
                }
                OriginKind::Project(source, member) => {
                    projection.push(member.as_str());
                    pending.push(Step::Pop);
                    pending.push(Step::Visit(source, read, receivers, acquiring));
                }
                OriginKind::ReceiverUse(source, source_read) => {
                    if let Some(source) = source {
                        pending.push(Step::Visit(
                            source,
                            if source_read.invocation {
                                reads.push(source_read, read)
                            } else {
                                reads.source(source_read, read, acquiring)
                            },
                            receivers,
                            acquiring,
                        ));
                    }
                }
                OriginKind::ReceiverContexts(source, contexts) => {
                    let contexts = receiver_contexts::reads(contexts, read, &mut reads, acquiring);
                    if contexts.is_empty() {
                        pending.push(Step::Visit(source, read, receivers, acquiring));
                    }
                    for read in contexts {
                        pending.push(Step::Visit(source, read, receivers, acquiring));
                    }
                }
                OriginKind::ReceiverAcquisition(source) => {
                    pending.push(Step::Visit(
                        source,
                        read,
                        receivers,
                        acquiring && projection.is_empty(),
                    ));
                }
                OriginKind::ReceiverSnapshot(source) => {
                    // Already acquired segments are frozen; later descendants
                    // still read the receiver at its caller's invocation.
                    pending.push(Step::Visit(
                        source,
                        read,
                        receivers && !projection.is_empty(),
                        acquiring,
                    ));
                }
                OriginKind::Receiver(source, scope, name, possible_only)
                    if receivers_only && receivers && (possible || !possible_only) =>
                {
                    if self.receiver_history_invalidated(
                        (source, *scope, name),
                        &projection,
                        (read, &reads),
                        possible,
                    ) {
                        return true;
                    }
                }
                OriginKind::Receiver(..) | OriginKind::ReceiverOwnedContexts(..) => {}
                origin @ OriginKind::InvocationReceiver { .. } => {
                    if !possible
                        && self.invocation_receiver_invalidated(
                            origin,
                            (file, source_context),
                            receivers_only,
                            receivers,
                            (read, &reads),
                            &projection,
                        )
                    {
                        return true;
                    }
                }
                OriginKind::Source(origin) => {
                    if !receivers_only
                        && self.source_origin_invalidated(
                            origin,
                            &projection,
                            hint_len,
                            file,
                            (offset, context),
                            &|writer| {
                                reads.offset(read, &origin.file, writer).or_absent(|| {
                                    (origin.file == file && writer == context).then_some(offset)
                                })
                            },
                        )
                    {
                        return true;
                    }
                }
            }
        }
        false
    }
}

#[path = "origin_invocation.rs"]
mod invocation;

#[path = "origin_receiver.rs"]
mod receiver;

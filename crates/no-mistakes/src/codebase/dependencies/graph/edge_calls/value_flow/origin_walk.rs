use super::*;

impl ValueFlowResolution<'_, '_> {
    pub(super) fn origins_invalidated(
        &self,
        file: &std::path::Path,
        offset: u32,
        context: Option<CallableId>,
        receivers_only: bool,
        hint: Option<&member_projection::MemberProjection<'_>>,
    ) -> bool {
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
        let mut reads = receiver_reads::ReceiverReads::default();
        let mut visited = fx_set();
        while let Some(step) = pending.pop() {
            let Step::Visit(node, read, receivers, acquiring) = step else {
                projection.pop();
                continue;
            };
            if (receivers_only && !node.has_receivers) || (!receivers_only && !node.has_defaults) {
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
                OriginKind::Receiver(source, scope, name) if receivers_only && receivers => {
                    if self.indexes.file(self.facts, source).is_some_and(|index| {
                        index
                            .values
                            .receiver_mutations
                            .as_ref()
                            .and_then(|receivers| receivers.get(scope))
                            .and_then(|names| names.get(name))
                            .is_some_and(|history| {
                                history.assignment_invalidated_reads(&projection, &|writer| {
                                    reads.offset(read, source, writer)
                                })
                            })
                    }) {
                        return true;
                    }
                }
                OriginKind::Receiver(..) | OriginKind::ReceiverOwnedContexts(..) => {}
                origin @ OriginKind::InvocationReceiver { .. } => {
                    if self.invocation_receiver_invalidated(
                        origin,
                        file,
                        context,
                        receivers_only,
                        receivers,
                        &projection,
                    ) {
                        return true;
                    }
                }
                OriginKind::Source(origin) => {
                    if !receivers_only
                        && self.source_origin_invalidated(
                            origin,
                            &projection,
                            file,
                            offset,
                            context,
                        )
                    {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn invocation_receiver_invalidated(
        &self,
        origin: &OriginKind,
        file: &std::path::Path,
        context: Option<CallableId>,
        receivers_only: bool,
        receivers: bool,
        projection: &[&str],
    ) -> bool {
        if self.invocation_read.is_some() {
            return false;
        }
        let OriginKind::InvocationReceiver {
            read,
            history_file,
            history,
        } = origin
        else {
            return false;
        };
        // Supplier offsets only order their own history. Reader ownership is
        // file-qualified, while member writes follow the receiver projection.
        read.file != *history_file
            || if receivers_only {
                receivers
                    && history.assignment_invalidated(projection, read.offset, read.context, false)
            } else {
                read.owner.0 != file || context != Some(read.owner.1)
            }
    }
}

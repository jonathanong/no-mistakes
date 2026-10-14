use super::*;
use receiver_reads::{ReadId, ReceiverReads};

/// Expand captured caller paths into links in the same walk-local arena.
pub(super) fn reads<'a>(
    root: &'a OriginNode,
    prior: ReadId,
    arena: &mut ReceiverReads<'a>,
    acquiring: bool,
) -> Vec<ReadId> {
    let mut pending = vec![(root, prior, acquiring)];
    let mut visited = fx_set();
    let mut reads = Vec::new();
    while let Some((node, read, acquiring)) = pending.pop() {
        if !visited.insert((node as *const OriginNode as usize, read, acquiring)) {
            continue;
        }
        match &node.kind {
            OriginKind::Union(left, right) => {
                pending.push((left, read, acquiring));
                pending.push((right, read, acquiring));
            }
            OriginKind::Project(source, _) | OriginKind::ReceiverSnapshot(source) => {
                pending.push((source, read, acquiring))
            }
            OriginKind::ReceiverContexts(_, contexts) => pending.push((contexts, read, acquiring)),
            OriginKind::ReceiverAcquisition(source) => pending.push((source, read, false)),
            OriginKind::ReceiverOwnedContexts(..) if !acquiring => reads.push(read),
            OriginKind::ReceiverOwnedContexts(root, file, owner) => {
                for frame in receiver_owned_contexts::frames(root, file, *owner) {
                    if let Some(frame) = frame {
                        pending.push((frame, None, acquiring));
                    } else {
                        reads.push(None);
                    }
                }
            }
            OriginKind::ReceiverUse(source, source_read) => {
                let read = if source_read.invocation {
                    arena.push(source_read, read)
                } else {
                    read
                };
                if let Some(source) = source {
                    pending.push((source, read, acquiring));
                } else {
                    reads.push(read);
                }
            }
            OriginKind::Source(_) if read.is_some() => reads.push(read),
            OriginKind::Source(_) | OriginKind::InvocationReceiver { .. } => {}
            OriginKind::Receiver(..) => reads.push(read),
        }
    }
    reads
}

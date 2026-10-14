use super::*;

/// Select acquisition frames lazily, retaining shared caller branches intact.
pub(super) fn frames<'a>(
    root: &'a OriginNode,
    file: &std::path::Path,
    owner: CallableId,
) -> Vec<Option<&'a OriginNode>> {
    let mut pending = vec![root];
    let mut visited = fx_set();
    let mut frames = Vec::new();
    let mut unmatched = false;
    while let Some(node) = pending.pop() {
        if !visited.insert(node as *const OriginNode as usize) {
            continue;
        }
        match &node.kind {
            OriginKind::Union(left, right) => {
                pending.push(left);
                pending.push(right);
            }
            OriginKind::Project(source, _)
            | OriginKind::ReceiverSnapshot(source)
            | OriginKind::ReceiverAcquisition(source) => pending.push(source),
            OriginKind::ReceiverContexts(_, contexts) => pending.push(contexts),
            OriginKind::ReceiverUse(source, read) => {
                if read.owns(file, Some(owner)) {
                    frames.push(Some(node));
                } else if let Some(source) = source {
                    pending.push(source);
                } else {
                    unmatched = true;
                }
            }
            OriginKind::ReceiverOwnedContexts(root, selected_file, selected_owner) => {
                for frame in self::frames(root, selected_file, *selected_owner) {
                    if let Some(frame) = frame {
                        pending.push(frame);
                    } else {
                        unmatched = true;
                    }
                }
            }
            // Default validation metadata is not an invocation alternative.
            OriginKind::Source(_) | OriginKind::InvocationReceiver { .. } => {}
            OriginKind::Receiver(..) => unmatched = true,
        }
    }
    if unmatched {
        frames.push(None);
    }
    frames
}

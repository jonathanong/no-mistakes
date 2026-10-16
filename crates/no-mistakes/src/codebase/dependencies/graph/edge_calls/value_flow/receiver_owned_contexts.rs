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

const FRAME_CAPACITY: usize = 32;
const SELECTOR_DEPTH: usize = 4;

/// Ephemeral traversal scratch; overflow delegates to the original full selector.
pub(super) struct StackFrames<'a> {
    frames: [Option<&'a OriginNode>; FRAME_CAPACITY],
    len: usize,
}
impl<'a> StackFrames<'a> {
    fn new() -> Self {
        Self {
            frames: [None; FRAME_CAPACITY],
            len: 0,
        }
    }
    fn push(&mut self, frame: Option<&'a OriginNode>) -> Option<()> {
        let slot = self.frames.get_mut(self.len)?;
        *slot = frame;
        self.len += 1;
        Some(())
    }
    fn pop(&mut self) -> Option<Option<&'a OriginNode>> {
        self.len = self.len.checked_sub(1)?;
        Some(self.frames[self.len])
    }
    pub(super) fn as_slice(&self) -> &[Option<&'a OriginNode>] {
        &self.frames[..self.len]
    }
}

pub(super) fn bounded_frames<'a>(
    root: &'a OriginNode,
    file: &std::path::Path,
    owner: CallableId,
) -> Option<StackFrames<'a>> {
    select_bounded(root, file, owner, 0)
}
fn select_bounded<'a>(
    root: &'a OriginNode,
    file: &std::path::Path,
    owner: CallableId,
    depth: usize,
) -> Option<StackFrames<'a>> {
    if depth > SELECTOR_DEPTH {
        return None;
    }
    let mut pending = StackFrames::new();
    pending.push(Some(root))?;
    let mut visited = StackFrames::new();
    let mut selected = StackFrames::new();
    let mut unmatched = false;
    while let Some(Some(node)) = pending.pop() {
        if visited
            .as_slice()
            .iter()
            .flatten()
            .any(|previous| std::ptr::eq(*previous, node))
        {
            continue;
        }
        visited.push(Some(node))?;
        match &node.kind {
            OriginKind::Union(left, right) => {
                pending.push(Some(left))?;
                pending.push(Some(right))?;
            }
            OriginKind::Project(source, _)
            | OriginKind::ReceiverSnapshot(source)
            | OriginKind::ReceiverAcquisition(source) => pending.push(Some(source))?,
            OriginKind::ReceiverContexts(_, contexts) => pending.push(Some(contexts))?,
            OriginKind::ReceiverUse(source, read) => {
                if read.owns(file, Some(owner)) {
                    selected.push(Some(node))?;
                } else if let Some(source) = source {
                    pending.push(Some(source))?;
                } else {
                    unmatched = true;
                }
            }
            OriginKind::ReceiverOwnedContexts(root, selected_file, selected_owner) => {
                // Each nested selector has independent ownership and pointer deduplication.
                let inner = select_bounded(root, selected_file, *selected_owner, depth + 1)?;
                for frame in inner.as_slice() {
                    if let Some(frame) = frame {
                        pending.push(Some(frame))?;
                    } else {
                        unmatched = true;
                    }
                }
            }
            OriginKind::Source(_) | OriginKind::InvocationReceiver { .. } => {}
            OriginKind::Receiver(..) => unmatched = true,
        }
    }
    if unmatched {
        selected.push(None)?;
    }
    Some(selected)
}

#[cfg(test)]
#[path = "receiver_owned_contexts/tests.rs"]
mod tests;

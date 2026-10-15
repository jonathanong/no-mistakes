use super::*;
use std::sync::Arc;

#[derive(Default)]
pub(super) struct CaptureInterner {
    nodes: FxHashMap<(nested::ReceiverRead, Option<usize>), Arc<Node>>,
    #[cfg(feature = "test-instrumentation")]
    pub(super) logical_tokens: usize,
}
struct Node {
    id: usize,
    read: nested::ReceiverRead,
    next: Option<Arc<Node>>,
}
impl Drop for Node {
    fn drop(&mut self) {
        // Long alias chains must not recurse when the last history is released.
        let mut next = self.next.take();
        while let Some(node) = next {
            match Arc::try_unwrap(node) {
                Ok(mut node) => next = node.next.take(),
                Err(_) => break,
            }
        }
    }
}
#[derive(Clone, Default)]
pub struct CaptureReads {
    head: Option<Arc<Node>>,
    len: usize,
}
impl CaptureInterner {
    pub(super) fn capture(&mut self, reads: &[nested::ReceiverRead]) -> CaptureReads {
        #[cfg(feature = "test-instrumentation")]
        {
            self.logical_tokens += reads.len();
        }
        let mut head: Option<Arc<Node>> = None;
        for read in reads.iter().rev() {
            let key = (*read, head.as_ref().map(|node| node.id));
            let next_id = self.nodes.len();
            head = Some(Arc::clone(self.nodes.entry(key).or_insert_with(|| {
                Arc::new(Node {
                    id: next_id,
                    read: *read,
                    next: head,
                })
            })));
        }
        CaptureReads {
            head,
            len: reads.len(),
        }
    }
    #[cfg(feature = "test-instrumentation")]
    pub(super) fn nodes(&self) -> usize {
        self.nodes.len()
    }
}
impl CaptureReads {
    pub(super) fn len(&self) -> usize {
        self.len
    }
    pub(super) fn iter(&self) -> impl Iterator<Item = &nested::ReceiverRead> {
        std::iter::successors(self.head.as_deref(), |node| node.next.as_deref())
            .map(|node| &node.read)
    }
}
impl PartialEq for CaptureReads {
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len && self.iter().eq(other.iter())
    }
}
impl Eq for CaptureReads {}
impl std::fmt::Debug for CaptureReads {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_list().entries(self.iter()).finish()
    }
}

#[test]
fn shared_capture_debug_preserves_unknown_reads_and_source_order() {
    let reads = [Some((7, None)), None, Some((19, Some(CallableId(3))))];
    let mut interner = CaptureInterner::default();
    let capture = interner.capture(&reads);
    assert_eq!(format!("{capture:?}"), format!("{:?}", reads.as_slice()));
    assert_eq!(capture.iter().copied().collect::<Vec<_>>(), reads);
    assert_eq!(format!("{:?}", interner.capture(&[])), "[]");
}

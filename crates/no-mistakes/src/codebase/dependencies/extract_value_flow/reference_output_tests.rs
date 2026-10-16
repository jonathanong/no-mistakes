use super::ReferenceOutput;
#[test]
fn empty_and_single_outputs_need_no_heap_buffer() {
    let mut output = ReferenceOutput::default();
    output.extend([]);
    assert!(output.first.is_none());
    assert_eq!(output.overflow.capacity(), 0);
    output.extend([7]);
    output.extend([]);
    assert_eq!(output.overflow.capacity(), 0);
    assert_eq!(output.into_iter().collect::<Vec<_>>(), [7]);
    assert_eq!(ReferenceOutput::<u8>::default().into_iter().count(), 0);
}
#[test]
fn multiple_extensions_keep_all_branches_in_order() {
    let mut output = ReferenceOutput::default();
    output.extend([1, 2]);
    output.extend([3, 4]);
    assert!(output.overflow.capacity() > 0);
    assert_eq!(output.into_iter().collect::<Vec<_>>(), [1, 2, 3, 4]);
}

impl<T> ReferenceOutput<T> {
    pub(in super::super) fn into_iter(self) -> impl Iterator<Item = T> {
        self.first.into_iter().chain(self.overflow)
    }
}

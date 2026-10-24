use super::{Memo, INLINE};

#[test]
fn shallow_pairs_stay_inline_and_overflow_keeps_earlier_pairs() {
    let mut memo = Memo::default();
    for id in 0..INLINE {
        assert!(!memo.contains((id, id + 1)));
        memo.insert((id, id + 1));
        assert!(memo.contains((id, id + 1)));
    }
    assert_eq!(
        memo.large.capacity(),
        0,
        "shallow comparisons allocate no map"
    );
    memo.insert((INLINE, INLINE + 1));
    assert_eq!(memo.large.len(), 1);
    for id in 0..=INLINE {
        assert!(memo.contains((id, id + 1)));
        assert!(
            !memo.contains((id + 1, id)),
            "pair direction remains explicit"
        );
    }
}

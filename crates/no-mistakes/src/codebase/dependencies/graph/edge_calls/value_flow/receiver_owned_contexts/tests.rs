use super::super::default_origins::{DefaultOriginUse, ReceiverRead};
use super::*;
use std::sync::Arc;

fn node(kind: OriginKind) -> Arc<OriginNode> {
    Arc::new(OriginNode {
        kind,
        height: 0,
        has_receivers: false,
        has_receiver_reads: false,
        has_defaults: false,
        has_unsealed_owned_contexts: false,
    })
}
fn read(owner: u32, source: Option<Arc<OriginNode>>) -> Arc<OriginNode> {
    node(OriginKind::ReceiverUse(
        source,
        ReceiverRead {
            file: "fixture.mts".into(),
            offset: 10,
            context: None,
            invocation: false,
            repeated: false,
            owned_context: Some(Box::new(("fixture.mts".into(), CallableId(owner), true))),
        },
    ))
}
fn unmatched() -> Arc<OriginNode> {
    node(OriginKind::Receiver(
        "fixture.mts".into(),
        0,
        "receiver".into(),
        false,
    ))
}
fn ignored() -> Arc<OriginNode> {
    node(OriginKind::Source(DefaultOriginUse {
        file: "fixture.mts".into(),
        scope: 0,
        name: "source".into(),
        origin: 0,
    }))
}
fn compare(root: &OriginNode, owner: u32) -> usize {
    let owner = CallableId(owner);
    let file = std::path::Path::new("fixture.mts");
    let expected = frames(root, file, owner);
    let actual = bounded_frames(root, file, owner).expect("within bounded capacity");
    let pointers = |frames: &[Option<&OriginNode>]| {
        frames
            .iter()
            .map(|frame| frame.map(|node| node as *const OriginNode as usize))
            .collect::<Vec<_>>()
    };
    assert_eq!(pointers(actual.as_slice()), pointers(&expected));
    actual.len
}

#[test]
fn bounded_selector_preserves_shared_dag_order_and_unmatched_sentinel() {
    let first = read(1, None);
    let second = read(1, None);
    let wrapped = node(OriginKind::Project(
        node(OriginKind::ReceiverSnapshot(node(
            OriginKind::ReceiverAcquisition(first.clone()),
        ))),
        "member".into(),
    ));
    let contexts = node(OriginKind::ReceiverContexts(unmatched(), second));
    let root = node(OriginKind::Union(
        node(OriginKind::Union(wrapped, first)),
        node(OriginKind::Union(
            contexts,
            node(OriginKind::Union(unmatched(), ignored())),
        )),
    ));
    assert_eq!(compare(&root, 1), 3);
    assert_eq!(compare(&root, 2), 1);
}

#[test]
fn nested_selectors_keep_independent_owner_and_pointer_selection() {
    let outer = read(2, None);
    let inner = read(1, Some(outer));
    let root = node(OriginKind::ReceiverOwnedContexts(
        node(OriginKind::Union(
            inner.clone(),
            node(OriginKind::Union(inner, unmatched())),
        )),
        "fixture.mts".into(),
        CallableId(1),
    ));
    assert_eq!(compare(&root, 1), 2);
    assert_eq!(compare(&root, 2), 2);
}

#[test]
fn empty_and_unmatched_are_distinct_and_unmatched_is_appended_once() {
    assert_eq!(compare(&ignored(), 1), 0);
    assert_eq!(compare(&unmatched(), 1), 1);
    let root = node(OriginKind::Union(unmatched(), unmatched()));
    assert_eq!(compare(&root, 1), 1);
}

#[test]
fn capacity_overflow_restarts_the_original_full_selector() {
    let mut root = read(1, None);
    for _ in 0..FRAME_CAPACITY {
        root = node(OriginKind::ReceiverSnapshot(root));
    }
    let file = std::path::Path::new("fixture.mts");
    assert!(bounded_frames(&root, file, CallableId(1)).is_none());
    assert_eq!(frames(&root, file, CallableId(1)).len(), 1);
}

#[test]
fn nested_depth_overflow_preserves_the_original_full_selector() {
    let mut root = read(1, None);
    for _ in 0..SELECTOR_DEPTH + 2 {
        root = node(OriginKind::ReceiverOwnedContexts(
            root,
            "fixture.mts".into(),
            CallableId(1),
        ));
    }
    let file = std::path::Path::new("fixture.mts");
    assert!(bounded_frames(&root, file, CallableId(1)).is_none());
    assert_eq!(frames(&root, file, CallableId(1)).len(), 1);
}

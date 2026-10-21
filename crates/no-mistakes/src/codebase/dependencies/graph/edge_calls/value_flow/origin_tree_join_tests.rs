use super::*;
use std::sync::Arc;

fn source(name: &str) -> OriginKind {
    OriginKind::Source(super::super::default_origins::DefaultOriginUse {
        file: std::path::PathBuf::from("a.ts"),
        scope: 0,
        name: name.into(),
        origin: 0,
    })
}

fn leaf(cache: &ParameterCache, name: &str) -> Arc<OriginNode> {
    cache.origin_node(source(name), 1)
}

fn union(cache: &ParameterCache, left: Arc<OriginNode>, right: Arc<OriginNode>) -> Arc<OriginNode> {
    let height = 1 + left.height.max(right.height);
    cache.origin_node(OriginKind::Union(left, right), height)
}

#[test]
fn join_shares_identical_arcs_and_rebalances_unbalanced_unions() {
    let cache = ParameterCache::default();
    let a = leaf(&cache, "a");
    let shared = join(Arc::clone(&a), Arc::clone(&a), &cache);
    assert!(Arc::ptr_eq(&shared, &a));
    assert!(a.has_defaults);
    assert!(!a.has_receivers);

    let balanced = join(leaf(&cache, "l"), leaf(&cache, "r"), &cache);
    assert_eq!(balanced.height, 2);
    assert!(matches!(balanced.kind, OriginKind::Union(_, _)));

    let ll = union(
        &cache,
        union(&cache, leaf(&cache, "a"), leaf(&cache, "b")),
        leaf(&cache, "c"),
    );
    let ll_joined = join(ll, leaf(&cache, "d"), &cache);
    assert_eq!(shape(&ll_joined), (3, 2, 2, names(["a", "b", "c", "d"])));

    let lr = union(
        &cache,
        leaf(&cache, "a"),
        union(&cache, leaf(&cache, "b"), leaf(&cache, "c")),
    );
    let lr_joined = join(lr, leaf(&cache, "d"), &cache);
    assert_eq!(shape(&lr_joined), (3, 2, 2, names(["a", "b", "c", "d"])));

    let rr = union(
        &cache,
        leaf(&cache, "a"),
        union(&cache, leaf(&cache, "b"), leaf(&cache, "c")),
    );
    let rr_joined = join(leaf(&cache, "d"), rr, &cache);
    assert_eq!(shape(&rr_joined), (3, 2, 2, names(["d", "a", "b", "c"])));

    let rl = union(
        &cache,
        union(&cache, leaf(&cache, "a"), leaf(&cache, "b")),
        leaf(&cache, "c"),
    );
    let rl_joined = join(leaf(&cache, "d"), rl, &cache);
    assert_eq!(shape(&rl_joined), (3, 2, 2, names(["d", "a", "b", "c"])));

    // Height-4 spines exercise `balance`'s single-rotation arm. Forcing that
    // arm closed walks `children` of a leaf and panics, and a swapped double
    // rotation changes the child heights below.
    let single_left = join(
        union(&cache, left_spine(&cache), leaf(&cache, "e")),
        leaf(&cache, "f"),
        &cache,
    );
    assert_eq!(
        shape(&single_left),
        (4, 3, 3, names(["p", "q", "r", "s", "e", "f"]))
    );

    let single_right = join(
        leaf(&cache, "x"),
        union(&cache, leaf(&cache, "a"), right_spine(&cache)),
        &cache,
    );
    assert_eq!(
        shape(&single_right),
        (4, 3, 3, names(["x", "a", "s", "p", "q", "r"]))
    );

    // The inner right child is taller, so this is a double rotation. Taking
    // the single-rotation arm instead leaves height 5.
    let double_right = join(
        leaf(&cache, "x"),
        union(&cache, leaf(&cache, "a"), left_spine(&cache)),
        &cache,
    );
    assert_eq!(
        shape(&double_right),
        (4, 3, 2, names(["x", "a", "p", "q", "r", "s"]))
    );

    let double_left = join(
        union(&cache, right_spine(&cache), leaf(&cache, "a")),
        leaf(&cache, "y"),
        &cache,
    );
    assert_eq!(
        shape(&double_left),
        (4, 3, 3, names(["s", "p", "q", "r", "a", "y"]))
    );
}

fn names<const N: usize>(values: [&str; N]) -> Vec<String> {
    values.into_iter().map(str::to_string).collect()
}

fn left_spine(cache: &ParameterCache) -> Arc<OriginNode> {
    let pair = union(cache, leaf(cache, "p"), leaf(cache, "q"));
    let three = union(cache, pair, leaf(cache, "r"));
    union(cache, three, leaf(cache, "s"))
}

fn right_spine(cache: &ParameterCache) -> Arc<OriginNode> {
    let pair = union(cache, leaf(cache, "p"), leaf(cache, "q"));
    let three = union(cache, pair, leaf(cache, "r"));
    union(cache, leaf(cache, "s"), three)
}

fn shape(node: &OriginNode) -> (usize, usize, usize, Vec<String>) {
    let OriginKind::Union(left, right) = &node.kind else {
        panic!("rebalanced join must stay a union");
    };
    (node.height, left.height, right.height, leaves(node))
}

fn leaves(node: &OriginNode) -> Vec<String> {
    match &node.kind {
        OriginKind::Source(source) => vec![source.name.clone()],
        OriginKind::Union(left, right) => {
            let mut names = leaves(left);
            names.extend(leaves(right));
            names
        }
        _ => panic!("origin join fixture produced a non-source leaf"),
    }
}

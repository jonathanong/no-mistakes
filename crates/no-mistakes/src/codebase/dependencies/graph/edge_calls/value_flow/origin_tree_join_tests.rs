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
    assert_eq!(ll_joined.height, 3);
    assert!(matches!(ll_joined.kind, OriginKind::Union(_, _)));

    let lr = union(
        &cache,
        leaf(&cache, "a"),
        union(&cache, leaf(&cache, "b"), leaf(&cache, "c")),
    );
    let lr_joined = join(lr, leaf(&cache, "d"), &cache);
    assert_eq!(lr_joined.height, 3);

    let rr = union(
        &cache,
        leaf(&cache, "a"),
        union(&cache, leaf(&cache, "b"), leaf(&cache, "c")),
    );
    let rr_joined = join(leaf(&cache, "d"), rr, &cache);
    assert_eq!(rr_joined.height, 3);

    let rl = union(
        &cache,
        union(&cache, leaf(&cache, "a"), leaf(&cache, "b")),
        leaf(&cache, "c"),
    );
    let rl_joined = join(leaf(&cache, "d"), rl, &cache);
    assert_eq!(rl_joined.height, 3);
}

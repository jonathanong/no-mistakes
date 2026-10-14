use super::*;
use std::sync::Arc;
// Persistent AVL unions keep fan-in balanced and fan-out shared. Projections
// retain one parent instead of rebuilding every origin in that parent.
pub(super) fn join(
    left: Arc<OriginNode>,
    right: Arc<OriginNode>,
    cache: &ParameterCache,
) -> Arc<OriginNode> {
    if Arc::ptr_eq(&left, &right) {
        return left;
    }
    if left.height > right.height + 1 {
        let (a, b) = children(&left);
        return balance(Arc::clone(a), join(Arc::clone(b), right, cache), cache);
    }
    if right.height > left.height + 1 {
        let (a, b) = children(&right);
        return balance(join(left, Arc::clone(a), cache), Arc::clone(b), cache);
    }
    branch(left, right, cache)
}
fn children(node: &OriginNode) -> (&Arc<OriginNode>, &Arc<OriginNode>) {
    let OriginKind::Union(left, right) = &node.kind else {
        unreachable!("only unions have a balancing height above one");
    };
    (left, right)
}
fn branch(
    left: Arc<OriginNode>,
    right: Arc<OriginNode>,
    cache: &ParameterCache,
) -> Arc<OriginNode> {
    let height = 1 + left.height.max(right.height);
    cache.origin_node(OriginKind::Union(left, right), height)
}
fn balance(
    left: Arc<OriginNode>,
    right: Arc<OriginNode>,
    cache: &ParameterCache,
) -> Arc<OriginNode> {
    if left.height > right.height + 1 {
        let (a, b) = children(&left);
        if a.height >= b.height {
            return branch(Arc::clone(a), branch(Arc::clone(b), right, cache), cache);
        }
        let (b1, b2) = children(b);
        return branch(
            branch(Arc::clone(a), Arc::clone(b1), cache),
            branch(Arc::clone(b2), right, cache),
            cache,
        );
    }
    if right.height > left.height + 1 {
        let (a, b) = children(&right);
        if b.height >= a.height {
            return branch(branch(left, Arc::clone(a), cache), Arc::clone(b), cache);
        }
        let (a1, a2) = children(a);
        return branch(
            branch(left, Arc::clone(a1), cache),
            branch(Arc::clone(a2), Arc::clone(b), cache),
            cache,
        );
    }
    branch(left, right, cache)
}
